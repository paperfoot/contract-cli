// ═══════════════════════════════════════════════════════════════════════════
// Render — build a ContractRenderData JSON sidecar, write it next to the
// templates in a temp directory, and shell out to `typst compile`.
//
// Designed to be pure: callers pre-load the Contract row + clauses + parties
// and pass them in. No DB access here.
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use chrono::NaiveDate;
use serde::Serialize;

use crate::clauses::{self, Pack};
use crate::db::{Client, Contract, ContractClauseRow, Issuer};
use crate::error::{AppError, Result};
use crate::typst_assets;

// ─── Wire format ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContractRenderData {
    pub kind: String,
    pub kind_label: String,
    pub number: String,
    pub title: String,
    pub effective_date_display: String,
    pub end_date_display: Option<String>,
    pub term_text: String,
    pub governing_law: String,
    pub jurisdiction_phrase: String,
    pub venue: Option<String>,
    pub status: String,
    pub draft_watermark: bool,
    pub fee_text: Option<String>,
    pub our_party: PartyData,
    pub their_party: PartyData,
    pub clauses: Vec<ClauseRenderData>,
    pub signature: SignatureBlock,
    pub logo: Option<String>,
    /// Free-form notes — kept on row, not rendered on the contract body.
    /// Templates may show it as an internal "Notes for our records" footer if
    /// they want; default templates do not.
    pub internal_notes: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartyData {
    pub role_label: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub company_no: Option<String>,
    pub address: Vec<String>,
    pub jurisdiction: Option<String>,
    pub email: Option<String>,
    pub attn: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClauseRenderData {
    pub number: i64,
    pub slug: String,
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SignatureBlock {
    pub our_label: String,
    pub our_name: String,
    pub our_signer_name: Option<String>,
    pub our_signer_title: Option<String>,
    pub our_signer_date: Option<String>,
    pub their_label: String,
    pub their_name: String,
    pub their_signer_name: Option<String>,
    pub their_signer_title: Option<String>,
    pub their_signer_date: Option<String>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn fmt_date(iso: &str) -> String {
    NaiveDate::parse_from_str(iso, "%Y-%m-%d")
        .map(|d| d.format("%-d %B %Y").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

fn kind_label(kind: &str, terms: &serde_json::Value) -> String {
    match kind {
        "consulting" => "Consulting Services Agreement".into(),
        "nda" => {
            if terms.get("mutuality").and_then(|v| v.as_str()) == Some("unilateral") {
                "Non-Disclosure Agreement".into()
            } else {
                "Mutual Non-Disclosure Agreement".into()
            }
        }
        "msa" => "Master Services Agreement".into(),
        "sow" => "Statement of Work".into(),
        "service" => "Service Agreement".into(),
        other => format!("{} Agreement", capitalize(other)),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn party_role_labels(kind: &str, terms: &serde_json::Value) -> (String, String) {
    match kind {
        "consulting" => ("Consultant".into(), "Client".into()),
        "msa" | "sow" => ("Provider".into(), "Client".into()),
        "service" => ("Provider".into(), "Customer".into()),
        "nda" => {
            let mutuality = terms.get("mutuality").and_then(|v| v.as_str()).unwrap_or("mutual");
            if mutuality == "mutual" {
                ("Party A".into(), "Party B".into())
            } else {
                let disclosing = terms.get("disclosing_side").and_then(|v| v.as_str()).unwrap_or("us");
                if disclosing == "us" {
                    ("Disclosing Party".into(), "Receiving Party".into())
                } else {
                    ("Receiving Party".into(), "Disclosing Party".into())
                }
            }
        }
        _ => ("Party A".into(), "Party B".into()),
    }
}

fn jurisdiction_phrase(law: &str) -> String {
    // Best-effort heuristic. Falls back to "the courts of <law>".
    let l = law.trim();
    let lower = l.to_lowercase();
    if lower.contains("singapore") {
        "the courts of Singapore".into()
    } else if lower.contains("england") || lower.contains("wales") || lower.contains("united kingdom") || lower == "uk" {
        "the courts of England and Wales".into()
    } else if lower.contains("delaware") {
        "the state and federal courts located in Delaware".into()
    } else if lower.contains("new york") {
        "the state and federal courts of New York".into()
    } else if lower.contains("hong kong") {
        "the courts of the Hong Kong Special Administrative Region".into()
    } else if lower.contains("germany") {
        "the courts of Frankfurt am Main, Germany".into()
    } else if lower.contains("france") {
        "the courts of Paris, France".into()
    } else {
        format!("the courts of {}", l)
    }
}

fn term_text(c: &Contract) -> String {
    if let Some(end) = &c.end_date {
        format!("until {}", fmt_date(end))
    } else if let Some(m) = c.term_months {
        if m % 12 == 0 {
            let years = m / 12;
            if years == 1 {
                "for one year from the Effective Date".into()
            } else {
                format!("for {} years from the Effective Date", years)
            }
        } else if m == 1 {
            "for one month from the Effective Date".into()
        } else {
            format!("for {} months from the Effective Date", m)
        }
    } else {
        "indefinitely until terminated as provided below".into()
    }
}

fn ip_assignment_text(terms: &serde_json::Value) -> String {
    let mode = terms.get("ip_assignment").and_then(|v| v.as_str()).unwrap_or("client");
    match mode {
        "client" => "All deliverables produced specifically for the Client under this engagement (the “Deliverables”) belong to the Client. The Consultant assigns to the Client, on payment of the relevant fees, all right, title, and interest in the Deliverables.".into(),
        "consultant" | "provider" => "The Consultant retains ownership of all deliverables. The Client receives a non-exclusive, perpetual, worldwide, royalty-free licence to use them for its internal business purposes.".into(),
        "shared" => "The parties jointly own the deliverables. Each party may use them for any purpose without accounting to the other.".into(),
        _ => "All deliverables produced specifically for the Client under this engagement belong to the Client, on payment of the relevant fees.".into(),
    }
}

fn fee_text(c: &Contract) -> Option<String> {
    let kind = c.fee_type.as_deref()?;
    let amt = c.fee_amount_minor?;
    let cur = c.fee_currency.clone().unwrap_or_default();
    let symbol = finance_core::money::currency_symbol(&cur);
    let amt_str = finance_core::money::MinorUnits(amt).format_number();
    let amt_display = if symbol.is_empty() {
        format!("{} {}", amt_str, cur)
    } else {
        format!("{}{}", symbol, amt_str)
    };
    let sched_phrase = match c.fee_schedule.as_deref() {
        Some("on-completion") => ", payable on completion of the Services",
        Some("on-milestone") => ", payable on completion of each milestone",
        Some("monthly") => ", invoiced monthly in arrears",
        Some("upon-invoice") => ", invoiced as work is performed",
        _ => "",
    };
    Some(match kind {
        "fixed" => format!("a fixed fee of {amt_display}{sched_phrase}"),
        "hourly" => format!("an hourly rate of {amt_display}/hour{sched_phrase}"),
        "daily" => format!("a daily rate of {amt_display}/day{sched_phrase}"),
        "retainer" => format!("a monthly retainer of {amt_display}{sched_phrase}"),
        _ => format!("{amt_display}{sched_phrase}"),
    })
}

fn deliverables_block(terms: &serde_json::Value) -> String {
    let items: Vec<String> = terms
        .get("deliverables")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if items.is_empty() {
        "_To be agreed in writing by the parties._".into()
    } else {
        items
            .into_iter()
            .map(|s| format!("- {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn party_display(issuer: &Issuer, role_label: &str) -> PartyData {
    PartyData {
        role_label: role_label.to_string(),
        display_name: issuer.name.clone(),
        legal_name: issuer.legal_name.clone(),
        company_no: issuer.company_no.clone(),
        address: issuer.address.clone(),
        jurisdiction: Some(issuer.jurisdiction.profile().country.to_string()),
        email: issuer.email.clone(),
        attn: None,
    }
}

fn client_display(client: &Client, role_label: &str) -> PartyData {
    PartyData {
        role_label: role_label.to_string(),
        display_name: client.name.clone(),
        legal_name: client.legal_name.clone(),
        company_no: client.company_no.clone(),
        address: client.address.clone(),
        jurisdiction: client.legal_jurisdiction.clone(),
        email: client.email.clone(),
        attn: client.attn.clone(),
    }
}

fn vars_from(
    contract: &Contract,
    issuer: &Issuer,
    client: &Client,
    terms: &serde_json::Value,
    our_role: &str,
    their_role: &str,
) -> BTreeMap<String, String> {
    let mut v = BTreeMap::new();
    let our_legal = issuer.legal_name.clone().unwrap_or_else(|| issuer.name.clone());
    let their_legal = client.legal_name.clone().unwrap_or_else(|| client.name.clone());
    v.insert("our_name".into(), issuer.name.clone());
    v.insert("our_legal_name".into(), our_legal);
    v.insert("our_role".into(), our_role.to_string());
    v.insert("their_name".into(), client.name.clone());
    v.insert("their_legal_name".into(), their_legal);
    v.insert("their_role".into(), their_role.to_string());
    v.insert("title".into(), contract.title.clone());
    v.insert("number".into(), contract.number.clone());
    v.insert("effective_date".into(), fmt_date(&contract.effective_date));
    v.insert(
        "end_date".into(),
        contract.end_date.as_deref().map(fmt_date).unwrap_or_default(),
    );
    v.insert("term_text".into(), term_text(contract));
    v.insert("governing_law".into(), contract.governing_law.clone());
    v.insert(
        "jurisdiction_phrase".into(),
        jurisdiction_phrase(&contract.governing_law),
    );
    v.insert(
        "venue".into(),
        contract.venue.clone().unwrap_or_default(),
    );
    // NDA specifics
    v.insert(
        "purpose".into(),
        terms
            .get("purpose")
            .and_then(|x| x.as_str())
            .unwrap_or("the matters described in the title")
            .to_string(),
    );
    v.insert(
        "mutuality".into(),
        terms
            .get("mutuality")
            .and_then(|x| x.as_str())
            .unwrap_or("mutual")
            .to_string(),
    );
    v.insert(
        "confidentiality_years".into(),
        terms
            .get("confidentiality_years")
            .and_then(|x| x.as_i64())
            .map(|n| n.to_string())
            .unwrap_or_else(|| "3".into()),
    );
    v.insert(
        "termination_notice_days".into(),
        terms
            .get("termination_notice_days")
            .and_then(|x| x.as_i64())
            .or_else(|| {
                contract
                    .terms_json
                    .parse::<serde_json::Value>()
                    .ok()
                    .and_then(|t| t.get("termination_notice_days").and_then(|x| x.as_i64()))
            })
            .map(|n| n.to_string())
            .unwrap_or_else(|| "30".into()),
    );
    // Consulting / SOW
    v.insert(
        "deliverables_block".into(),
        deliverables_block(terms),
    );
    v.insert("ip_assignment_text".into(), ip_assignment_text(terms));
    v.insert(
        "fee_text".into(),
        fee_text(contract).unwrap_or_else(|| "as separately agreed in writing".into()),
    );
    v
}

// ─── Pipeline ────────────────────────────────────────────────────────────

pub fn build_render_data(
    contract: &Contract,
    issuer: &Issuer,
    client: &Client,
    clause_rows: &[ContractClauseRow],
    pack: &Pack,
    force_draft: bool,
    force_final: bool,
) -> Result<ContractRenderData> {
    let terms: serde_json::Value = serde_json::from_str(&contract.terms_json)
        .map_err(|e| AppError::Other(format!("invalid terms_json: {e}")))?;
    let (our_role, their_role) = party_role_labels(&contract.kind, &terms);

    let vars = vars_from(contract, issuer, client, &terms, &our_role, &their_role);

    // Resolve clauses through pack with optional overrides on each row.
    let included: Vec<String> = clause_rows.iter().map(|r| r.slug.clone()).collect();
    let mut overrides: BTreeMap<String, (Option<String>, Option<String>)> = BTreeMap::new();
    for r in clause_rows {
        if r.heading.is_some() || r.body.is_some() {
            overrides.insert(r.slug.clone(), (r.heading.clone(), r.body.clone()));
        }
    }
    let resolved = clauses::resolve(pack, &included, &overrides, &vars)?;
    let render_clauses: Vec<ClauseRenderData> = resolved
        .into_iter()
        .enumerate()
        .map(|(i, c)| ClauseRenderData {
            number: (i + 1) as i64,
            slug: c.slug,
            heading: c.heading,
            body: c.body,
        })
        .collect();

    let our_legal = issuer
        .legal_name
        .clone()
        .unwrap_or_else(|| issuer.name.clone());
    let their_legal = client
        .legal_name
        .clone()
        .unwrap_or_else(|| client.name.clone());

    let signature = SignatureBlock {
        our_label: our_role.clone(),
        our_name: our_legal,
        our_signer_name: contract.signed_by_us_name.clone(),
        our_signer_title: contract.signed_by_us_title.clone(),
        our_signer_date: contract.signed_by_us_at.as_deref().map(fmt_date),
        their_label: their_role.clone(),
        their_name: their_legal,
        their_signer_name: contract.signed_by_them_name.clone(),
        their_signer_title: contract.signed_by_them_title.clone(),
        their_signer_date: contract.signed_by_them_at.as_deref().map(fmt_date),
    };

    // DRAFT watermark logic: implicit unless contract is signed/active, but
    // user can force either direction.
    let is_executed = matches!(contract.status.as_str(), "signed" | "active");
    let draft = if force_draft {
        true
    } else if force_final {
        false
    } else {
        !is_executed
    };

    Ok(ContractRenderData {
        kind: contract.kind.clone(),
        kind_label: kind_label(&contract.kind, &terms),
        number: contract.number.clone(),
        title: contract.title.clone(),
        effective_date_display: fmt_date(&contract.effective_date),
        end_date_display: contract.end_date.as_deref().map(fmt_date),
        term_text: term_text(contract),
        governing_law: contract.governing_law.clone(),
        jurisdiction_phrase: jurisdiction_phrase(&contract.governing_law),
        venue: contract.venue.clone(),
        status: contract.status.clone(),
        draft_watermark: draft,
        fee_text: fee_text(contract),
        our_party: party_display(issuer, &our_role),
        their_party: client_display(client, &their_role),
        clauses: render_clauses,
        signature,
        logo: None, // populated by render_to_pdf if issuer has a logo
        internal_notes: contract.notes.clone(),
    })
}

pub fn render_to_pdf(
    template: &str,
    data: &mut ContractRenderData,
    issuer: &Issuer,
    out_path: &Path,
) -> Result<()> {
    typst_assets::ensure_extracted()?;
    if !typst_assets::has_template(template)? {
        return Err(AppError::InvalidInput(format!(
            "template '{template}' not found. Run: contract template list"
        )));
    }

    let tmp = tempfile::Builder::new()
        .prefix("contract-cli-render-")
        .tempdir()?;
    let root = tmp.path();
    copy_dir_contents(&typst_assets::project_root()?, root)?;

    // Copy logo (if any) into shared/ and set the json path.
    data.logo = stage_logo(root, issuer)?;

    let json_path = root.join("shared").join("contract.json");
    std::fs::write(&json_path, serde_json::to_vec_pretty(&data)?)?;

    let template_path = root.join("templates").join(format!("{template}.typ"));
    let status = Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(root)
        .arg(&template_path)
        .arg(out_path)
        .status()
        .map_err(|e| AppError::Render(format!("typst binary not found: {e}")))?;
    if !status.success() {
        return Err(AppError::Render(format!(
            "typst compile exited with {}",
            status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn stage_logo(root: &Path, issuer: &Issuer) -> Result<Option<String>> {
    let Some(src_raw) = &issuer.logo_path else {
        return Ok(None);
    };
    let src_expanded = expand_tilde(src_raw);
    let src = Path::new(&src_expanded);
    if !src.exists() {
        eprintln!(
            "warning: logo '{}' not found for issuer '{}' — rendering without",
            src.display(),
            issuer.slug
        );
        return Ok(None);
    }
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let rel = format!("shared/logo-{}.{ext}", issuer.slug);
    let dst = root.join(&rel);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, &dst)?;
    Ok(Some(format!("/{rel}")))
}

pub fn expand_tilde(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    s.to_string()
}

pub fn default_output_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join("Documents").join("Contracts")
}
