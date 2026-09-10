use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use chrono::{Datelike, NaiveDate};
use serde_json::{Value, json};

use crate::clauses;
use crate::cli::{ContractCmd, ContractListArgs, ContractNewArgs, ContractRenderArgs, SignArgs};
use crate::db::{self, Contract, ContractClauseRow};
use crate::error::{AppError, Result};
use crate::output::{Ctx, print_success};
use crate::render;

pub fn run(cmd: ContractCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        ContractCmd::New(args) => cmd_new(args, ctx),
        ContractCmd::List(args) => cmd_list(args, ctx),
        ContractCmd::Show { number } => cmd_show(&number, ctx),
        ContractCmd::Edit(args) => cmd_edit(args, ctx),
        ContractCmd::Render(args) => cmd_render(args, ctx),
        ContractCmd::Mark { number, status } => cmd_mark(&number, &status, ctx),
        ContractCmd::Sign(args) => cmd_sign(args, ctx),
        ContractCmd::Clauses(cmd) => super::clauses::run(cmd, ctx),
        ContractCmd::Duplicate(args) => cmd_duplicate(&args.number, args.client, args.r#as, ctx),
        ContractCmd::Delete(args) => cmd_delete(&args.number, args.force, ctx),
    }
}

// ─── new ─────────────────────────────────────────────────────────────────

fn cmd_new(args: ContractNewArgs, ctx: Ctx) -> Result<()> {
    let kind_names = crate::kinds::names();
    if !kind_names.contains(&args.kind.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "unknown kind '{}'. Expected one of: {}",
            args.kind,
            kind_names.join(", ")
        )));
    }
    let effective_iso = match args.effective {
        Some(s) => parse_date(&s)?,
        None => chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };
    let end_iso = match args.end {
        Some(s) => Some(parse_date(&s)?),
        None => None,
    };
    let term_months = match (args.term_months, args.term_years) {
        (Some(_), Some(_)) => {
            return Err(AppError::InvalidInput(
                "pass at most one of --term-months / --term-years".into(),
            ));
        }
        (Some(m), None) => Some(m),
        (None, Some(y)) => Some(
            y.checked_mul(12)
                .ok_or_else(|| AppError::InvalidInput("term years is too large".into()))?,
        ),
        (None, None) => None,
    };
    crate::legal::validate_dates(&effective_iso, end_iso.as_deref())?;
    let selected_law = crate::legal::select_profile(
        args.legal_profile.as_deref(),
        args.us_state.as_deref(),
        args.governing_law.as_deref(),
        args.venue.as_deref(),
    )?;
    if let Some(m) = term_months
        && m <= 0
    {
        return Err(AppError::InvalidInput(format!(
            "invalid term length {m} — must be a positive number of months"
        )));
    }
    if end_iso.is_some() && term_months.is_some() {
        return Err(AppError::InvalidInput(
            "--end and --term-months/--term-years are mutually exclusive".into(),
        ));
    }
    if let Some(t) = &args.template
        && !crate::typst_assets::has_template(t)?
    {
        return Err(AppError::InvalidInput(format!(
            "template '{t}' not found. Run: contract template list"
        )));
    }

    // Parse fee
    let (fee_type, fee_amount_minor, fee_currency) = match args.fee.as_deref() {
        None => (None, None, None),
        Some(spec) => {
            let (t, a, c) = parse_fee(spec)?;
            (Some(t), Some(a), Some(c))
        }
    };
    let fee_schedule = match args.fee_schedule.as_deref() {
        Some(v) => Some(validate_choice(
            "--fee-schedule",
            v,
            &["on-completion", "monthly", "on-milestone", "upon-invoice"],
        )?),
        None => None,
    };

    // Build terms_json
    let mut terms_obj = serde_json::Map::new();
    if let Some(p) = &args.purpose {
        terms_obj.insert("purpose".into(), Value::String(p.clone()));
    }
    if let Some(n) = args.termination_notice_days {
        terms_obj.insert("termination_notice_days".into(), json!(n));
    }
    if matches!(args.kind.as_str(), "nda" | "ncnda") {
        let mutuality = match args.mutuality.as_deref() {
            Some(v) => validate_choice("--mutuality", v, &["mutual", "unilateral"])?,
            None => "mutual".into(),
        };
        let disclosing = match args.disclosing_side.as_deref() {
            Some(v) => validate_choice("--disclosing-side", v, &["us", "them", "both"])?,
            None => {
                if mutuality == "unilateral" {
                    "us".into()
                } else {
                    "both".into()
                }
            }
        };
        terms_obj.insert("mutuality".into(), Value::String(mutuality));
        terms_obj.insert("disclosing_side".into(), Value::String(disclosing));
        terms_obj.insert("confidentiality_years".into(), json!(3));
    } else if matches!(args.kind.as_str(), "consulting" | "msa" | "sow" | "service") {
        if !args.deliverables.is_empty() {
            terms_obj.insert(
                "deliverables".into(),
                Value::Array(
                    args.deliverables
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if let Some(ip) = args.ip_assignment.as_deref() {
            let ip = validate_choice(
                "--ip-assignment",
                ip,
                &["client", "consultant", "provider", "shared"],
            )?;
            terms_obj.insert("ip_assignment".into(), Value::String(ip));
        }
        terms_obj
            .entry("confidentiality_years".to_string())
            .or_insert(json!(3));
    }
    // Free-form --term key=value pairs (the extension point for loan / ncnda
    // pack {{vars}} like principal_text, repayment_date, interest_text).
    for spec in &args.terms {
        let (k, v) = parse_term_kv(spec)?;
        terms_obj.insert(k, Value::String(v));
    }
    if let Some(profile) = &args.legal_profile {
        terms_obj.insert("legal_profile".into(), json!(profile));
    }
    crate::legal::validate_terms(&mut terms_obj)?;
    if args.kind == "ncnda"
        && terms_obj.get("mutuality").and_then(Value::as_str) == Some("unilateral")
    {
        return Err(AppError::InvalidInput(
            "the NCNDA pack is mutual; use nda for a unilateral disclosure agreement".into(),
        ));
    }
    let terms_json = Value::Object(terms_obj).to_string();

    // All pure input validation is done — only now touch the database.
    let mut conn = db::open()?;

    // Resolve client first — needed for default issuer
    let client = db::client_by_slug(&conn, &args.client)?;
    // Resolve issuer: --as > client.default_issuer > config.default_issuer
    let issuer_slug = args
        .r#as
        .or(client.default_issuer_slug.clone())
        .or_else(|| crate::config::load().ok().and_then(|c| c.default_issuer))
        .ok_or_else(|| {
            AppError::InvalidInput(
                "no issuer — pass --as <slug>, pin a default on the client, or set config.default_issuer".into(),
            )
        })?;
    let issuer = db::issuer_by_slug(&conn, &issuer_slug)?;

    let governing_law = selected_law.unwrap_or_else(|| {
        let country = issuer.jurisdiction.profile().country;
        if country == "United Kingdom" {
            "England and Wales".into()
        } else {
            country.to_string()
        }
    });
    crate::legal::validate_law(&governing_law, args.venue.as_deref())?;
    let venue = args.venue;

    let title = args
        .title
        .unwrap_or_else(|| default_title(&args.kind, &issuer.name, &client.name));

    // Pick clause pack (default: "standard")
    let pack_slug = args.pack.clone().unwrap_or_else(|| "standard".to_string());
    let pack = clauses::load_pack(&args.kind, &pack_slug)?;

    // Build the included clause list with --include / --exclude
    let mut included: Vec<String> = pack.pack.default_clauses.clone();
    let mut seen: BTreeSet<String> = included.iter().cloned().collect();
    for slug in &args.include {
        if !pack.clauses.contains_key(slug) {
            return Err(AppError::NotFound(format!(
                "clause '{slug}' is not defined in pack '{}/{pack_slug}'",
                args.kind
            )));
        }
        if seen.insert(slug.clone()) {
            included.push(slug.clone());
        }
    }
    for slug in &args.exclude {
        if !pack.clauses.contains_key(slug) {
            return Err(AppError::InvalidInput(format!(
                "unknown excluded clause {slug}"
            )));
        }
        included.retain(|s| s != slug);
    }
    let clause_rows: Vec<ContractClauseRow> = included
        .iter()
        .enumerate()
        .map(|(i, slug)| ContractClauseRow {
            id: 0,
            contract_id: 0,
            position: i as i64,
            slug: slug.clone(),
            heading: pack.clauses.get(slug).map(|d| d.heading.clone()),
            body: pack.clauses.get(slug).map(|d| d.body.clone()),
        })
        .collect();

    // Generate number
    let year = NaiveDate::parse_from_str(&effective_iso, "%Y-%m-%d")
        .map(|d| d.year())
        .unwrap_or_else(|_| chrono::Local::now().year());
    let number = db::next_contract_number(&conn, &issuer, year, &args.kind)?;

    let contract = Contract {
        id: 0,
        number,
        kind: args.kind.clone(),
        issuer_id: issuer.id,
        client_id: client.id,
        title,
        effective_date: effective_iso,
        end_date: end_iso,
        term_months,
        governing_law,
        venue,
        status: "draft".into(),
        sent_at: None,
        signed_at: None,
        terminated_at: None,
        notes: args.notes,
        fee_type,
        fee_amount_minor,
        fee_currency,
        fee_schedule,
        terms_json,
        clause_pack: pack_slug,
        clause_pack_version: pack.pack.version.clone(),
        default_template: args.template,
        signed_by_us_name: None,
        signed_by_us_title: None,
        signed_by_us_at: None,
        signed_by_them_name: None,
        signed_by_them_title: None,
        signed_by_them_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    };

    let _id = db::contract_create(&mut conn, &contract, &clause_rows)?;
    let saved = db::contract_get(&conn, &contract.number)?;
    print_success(ctx, &saved, |c| {
        println!(
            "created {} contract '{}' for {} ({} clauses)",
            c.kind,
            c.number,
            c.title,
            clause_rows.len()
        );
    });
    Ok(())
}

fn default_title(kind: &str, issuer_name: &str, client_name: &str) -> String {
    match kind {
        "nda" => format!("NDA — {issuer_name} & {client_name}"),
        "ncnda" => format!("NCNDA — {issuer_name} & {client_name}"),
        "consulting" => format!("Consulting Agreement — {issuer_name} × {client_name}"),
        "msa" => format!("Master Services Agreement — {issuer_name} & {client_name}"),
        "sow" => format!("Statement of Work — {issuer_name} × {client_name}"),
        "service" => format!("Service Agreement — {issuer_name} for {client_name}"),
        "loan" => format!("Loan Agreement — {issuer_name} & {client_name}"),
        _ => format!("Agreement — {issuer_name} & {client_name}"),
    }
}

fn parse_date(s: &str) -> Result<String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.format("%Y-%m-%d").to_string())
        .map_err(|_| AppError::InvalidInput(format!("invalid date '{s}' — expected YYYY-MM-DD")))
}

fn parse_fee(spec: &str) -> Result<(String, i64, String)> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidInput(format!(
            "fee spec '{spec}' — expected type:amount:currency (e.g. fixed:8400:SGD)"
        )));
    }
    let kind = parts[0].to_lowercase();
    if !matches!(kind.as_str(), "fixed" | "hourly" | "daily" | "retainer") {
        return Err(AppError::InvalidInput(format!(
            "unknown fee type '{kind}' (expected fixed | hourly | daily | retainer)"
        )));
    }
    if let Some((_, cadence)) = parts[2].split_once('/')
        && (kind != "retainer" || cadence != "month")
    {
        return Err(AppError::InvalidInput(
            "only retainer fees support the /month suffix".into(),
        ));
    }
    // Strip a "/month"-style cadence suffix from the currency segment; the
    // cadence belongs in --fee-schedule, not the currency code.
    let currency = parts[2]
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_uppercase();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::InvalidInput(format!(
            "invalid currency '{}' — expected a 3-letter ISO code (e.g. SGD, GBP, USD)",
            parts[2]
        )));
    }
    let amount_minor = parse_amount_minor(parts[1])?;
    Ok((kind, amount_minor, currency))
}

/// Parse a decimal money amount into integer minor units with checked
/// arithmetic — no f64, so no silent overflow saturation or cent drift.
/// Accepts "8400", "8400.5", "8400.50"; rejects sub-cent precision,
/// exponents, separators, zero, and negatives.
fn parse_amount_minor(text: &str) -> Result<i64> {
    let bad = |why: &str| AppError::InvalidInput(format!("invalid fee amount '{text}': {why}"));
    let t = text.trim();
    let (int_part, frac_part) = match t.split_once('.') {
        Some((i, f)) => (i, f),
        None => (t, ""),
    };
    if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(bad("must be a plain positive number like 8400 or 8400.50"));
    }
    if frac_part.len() > 2 || !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(bad("at most two decimal places (whole cents)"));
    }
    let whole: i64 = int_part.parse().map_err(|_| bad("too large"))?;
    let cents: i64 = match frac_part.len() {
        0 => 0,
        1 => frac_part.parse::<i64>().unwrap_or(0) * 10,
        _ => frac_part.parse::<i64>().unwrap_or(0),
    };
    let minor = whole
        .checked_mul(100)
        .and_then(|w| w.checked_add(cents))
        .ok_or_else(|| bad("too large"))?;
    if minor <= 0 {
        return Err(bad("must be positive"));
    }
    Ok(minor)
}

/// Validate a repeatable --term key=value flag into (key, value).
fn parse_term_kv(spec: &str) -> Result<(String, String)> {
    let (k, v) = spec.split_once('=').ok_or_else(|| {
        AppError::InvalidInput(format!(
            "invalid --term '{spec}' — expected key=value (e.g. repayment_date=2026-12-01)"
        ))
    })?;
    let k = k.trim();
    if k == "legal_profile" {
        return Err(AppError::InvalidInput(
            "use --legal-profile to select a jurisdiction profile".into(),
        ));
    }
    if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AppError::InvalidInput(format!(
            "invalid --term key '{k}' — use snake_case letters/digits"
        )));
    }
    Ok((k.to_string(), v.to_string()))
}

/// Enum-flag validation: lowercase + membership check with a helpful error.
fn validate_choice(flag: &str, value: &str, allowed: &[&str]) -> Result<String> {
    let v = value.trim().to_lowercase();
    if allowed.contains(&v.as_str()) {
        Ok(v)
    } else {
        Err(AppError::InvalidInput(format!(
            "invalid {flag} '{value}' (expected one of: {})",
            allowed.join(" | ")
        )))
    }
}

// ─── list ─────────────────────────────────────────────────────────────────

fn cmd_list(args: ContractListArgs, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let rows = db::contract_list(
        &conn,
        args.kind.as_deref(),
        args.status.as_deref(),
        args.issuer.as_deref(),
    )?;
    print_success(ctx, &rows, |rs| {
        if rs.is_empty() {
            println!("(no contracts — create one with: contract new --kind nda --client X)");
        } else {
            for c in rs {
                println!(
                    "  {:<28} {:<10} {:<8} {}",
                    c.number, c.kind, c.status, c.title
                );
            }
        }
    });
    Ok(())
}

// ─── show ─────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct ContractView<'a> {
    #[serde(flatten)]
    contract: &'a Contract,
    clauses: Vec<ContractClauseRow>,
}

fn cmd_show(number: &str, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let c = db::contract_get_or_404(&conn, number)?;
    let clauses = db::clauses_for(&conn, c.id)?;
    let view = ContractView {
        contract: &c,
        clauses: clauses.clone(),
    };
    print_success(ctx, &view, |v| {
        println!("Contract: {}", v.contract.number);
        println!("  Kind:           {}", v.contract.kind);
        println!("  Title:          {}", v.contract.title);
        println!("  Status:         {}", v.contract.status);
        println!("  Effective:      {}", v.contract.effective_date);
        if let Some(e) = &v.contract.end_date {
            println!("  End:            {}", e);
        }
        if let Some(t) = v.contract.term_months {
            println!("  Term:           {} months", t);
        }
        println!("  Governing law:  {}", v.contract.governing_law);
        if let Some(fee) = &v.contract.fee_type {
            println!(
                "  Fee:            {} {} {}",
                fee,
                v.contract
                    .fee_amount_minor
                    .map(|m| (m as f64) / 100.0)
                    .unwrap_or(0.0),
                v.contract.fee_currency.clone().unwrap_or_default()
            );
        }
        println!(
            "  Pack:           {} v{}",
            v.contract.clause_pack, v.contract.clause_pack_version
        );
        println!("  Clauses ({}):", v.clauses.len());
        for cl in &v.clauses {
            println!("    {:>2}. {}", cl.position + 1, cl.slug);
        }
    });
    Ok(())
}

// ─── edit ─────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn cmd_edit(args: crate::cli::ContractEditArgs, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let number = args.number;
    let mut c = db::contract_get_or_404(&conn, &number)?;
    if let Some(slug) = args.client {
        c.client_id = db::client_by_slug(&conn, &slug)?.id;
    }
    if let Some(v) = args.title {
        c.title = v;
    }
    if let Some(v) = args.effective {
        c.effective_date = parse_date(&v)?;
    }
    if let Some(v) = args.end {
        c.end_date = Some(parse_date(&v)?);
        c.term_months = None;
    }
    if let Some(v) = args.term_months {
        if v <= 0 {
            return Err(AppError::InvalidInput(format!(
                "invalid term length {v} — must be a positive number of months"
            )));
        }
        c.term_months = Some(v);
        c.end_date = None;
    }
    let selected = crate::legal::select_profile(
        args.legal_profile.as_deref(),
        args.us_state.as_deref(),
        args.governing_law.as_deref(),
        args.venue.as_deref(),
    )?;
    if let Some(v) = selected {
        c.governing_law = v;
    }
    if let Some(v) = args.venue {
        c.venue = Some(v);
    }
    if let Some(v) = args.fee {
        let (t, a, cur) = parse_fee(&v)?;
        c.fee_type = Some(t);
        c.fee_amount_minor = Some(a);
        c.fee_currency = Some(cur);
    }
    if let Some(v) = args.fee_schedule.as_deref() {
        c.fee_schedule = Some(validate_choice(
            "--fee-schedule",
            v,
            &["on-completion", "monthly", "on-milestone", "upon-invoice"],
        )?);
    }
    if !args.terms.is_empty() || args.legal_profile.is_some() {
        let mut terms: serde_json::Map<String, Value> = c
            .terms_json
            .parse::<Value>()
            .ok()
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        for spec in &args.terms {
            let (k, v) = parse_term_kv(spec)?;
            terms.insert(k, Value::String(v));
        }
        if let Some(profile) = args.legal_profile {
            terms.insert("legal_profile".into(), json!(profile));
        }
        crate::legal::validate_terms(&mut terms)?;
        c.terms_json = Value::Object(terms).to_string();
    }
    if let Some(v) = args.notes {
        c.notes = Some(v);
    }
    if let Some(v) = args.template {
        if !crate::typst_assets::has_template(&v)? {
            return Err(AppError::InvalidInput(format!(
                "template '{v}' not found. Run: contract template list"
            )));
        }
        c.default_template = Some(v);
    }
    let current_terms: Value = serde_json::from_str(&c.terms_json)?;
    if c.kind == "ncnda"
        && current_terms.get("mutuality").and_then(Value::as_str) == Some("unilateral")
    {
        return Err(AppError::InvalidInput(
            "the NCNDA pack is mutual; use nda for a unilateral disclosure agreement".into(),
        ));
    }
    if let Some(profile) = current_terms.get("legal_profile").and_then(Value::as_str) {
        crate::legal::select_profile(
            Some(profile),
            if profile == "us" {
                Some(c.governing_law.as_str())
            } else {
                None
            },
            Some(&c.governing_law),
            c.venue.as_deref(),
        )?;
    }
    crate::legal::validate_dates(&c.effective_date, c.end_date.as_deref())?;
    crate::legal::validate_law(&c.governing_law, c.venue.as_deref())?;
    db::contract_update_draft(&conn, &c)?;
    let saved = db::contract_get(&conn, &number)?;
    print_success(ctx, &saved, |s| println!("updated draft '{}'", s.number));
    Ok(())
}

// ─── render ───────────────────────────────────────────────────────────────

fn cmd_render(args: ContractRenderArgs, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let c = db::contract_get_or_404(&conn, &args.number)?;
    let issuer = db::issuer_list(&conn)?
        .into_iter()
        .find(|i| i.id == c.issuer_id)
        .ok_or_else(|| AppError::NotFound(format!("issuer #{}", c.issuer_id)))?;
    let client = db::client_list(&conn)?
        .into_iter()
        .find(|x| x.id == c.client_id)
        .ok_or_else(|| AppError::NotFound(format!("client #{}", c.client_id)))?;
    let clause_rows = db::clauses_for(&conn, c.id)?;
    let pack = clauses::load_pack_version(&c.kind, &c.clause_pack, &c.clause_pack_version)?;

    let template = args
        .template
        .or_else(|| c.default_template.clone())
        .or_else(|| {
            crate::config::load()
                .ok()
                .map(|c| c.default_template)
                .filter(|t| crate::typst_assets::has_template(t).unwrap_or(false))
        })
        .unwrap_or_else(|| "helvetica-nera".to_string());

    let out_path: PathBuf = match args.out {
        Some(p) => PathBuf::from(render::expand_tilde(&p)),
        None => {
            let dir = issuer
                .default_output_dir
                .clone()
                .map(|s| PathBuf::from(render::expand_tilde(&s)))
                .unwrap_or_else(render::default_output_dir);
            std::fs::create_dir_all(&dir)?;
            let safe_number: String = c
                .number
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect();
            dir.join(format!("{safe_number}.pdf"))
        }
    };

    let mut data = render::build_render_data(
        &c,
        &issuer,
        &client,
        &clause_rows,
        &pack,
        args.draft,
        args.final_render,
    )?;
    data.paper = args.paper;
    render::render_to_pdf(&template, &mut data, &issuer, &out_path)?;

    if args.open {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = Command::new(opener).arg(&out_path).status();
    }

    #[derive(serde::Serialize)]
    struct Out {
        number: String,
        template: String,
        out: String,
        draft_watermark: bool,
    }
    let report = Out {
        number: c.number.clone(),
        template,
        out: out_path.display().to_string(),
        draft_watermark: data.draft_watermark,
    };
    print_success(ctx, &report, |r| {
        let mark = if r.draft_watermark { " (DRAFT)" } else { "" };
        println!("rendered → {}{}", r.out, mark);
    });
    Ok(())
}

// ─── mark / sign ──────────────────────────────────────────────────────────

const STATUSES: &[&str] = &["draft", "sent", "signed", "active", "expired", "terminated"];

/// Legal lifecycle moves, as explicit edges. sent→draft recalls an unsigned
/// draft; everything after signing only moves forward — reopening an executed
/// contract would unlock clause edits on a document the counterparty already
/// signed. No stage-skipping (a draft cannot jump straight to active).
fn transition_allowed(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        ("draft", "sent" | "signed")
            | ("sent", "draft" | "signed")
            | ("signed", "active" | "expired" | "terminated")
            | ("active", "expired" | "terminated")
    )
}

fn cmd_mark(number: &str, status: &str, ctx: Ctx) -> Result<()> {
    let status = validate_choice("status", status, STATUSES)?;
    let mut connection = db::open()?;
    let conn = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let current = db::contract_get_or_404(&conn, number)?;
    if !transition_allowed(&current.status, &status) {
        return Err(AppError::InvalidInput(format!(
            "cannot mark '{}' {} → {} (executed contracts only move forward: signed → active → expired/terminated)",
            number, current.status, status
        )));
    }
    if status == "draft"
        && (current.signed_by_us_name.is_some() || current.signed_by_them_name.is_some())
    {
        return Err(AppError::InvalidInput(
            "a partially signed contract cannot be reopened; duplicate it as a new draft".into(),
        ));
    }
    db::contract_set_status(&conn, number, &status)?;
    let c = db::contract_get(&conn, number)?;
    conn.commit()?;
    print_success(ctx, &c, |c| println!("'{}' → {}", c.number, c.status));
    Ok(())
}

fn cmd_sign(args: SignArgs, ctx: Ctx) -> Result<()> {
    let side = validate_choice("--side", &args.side, &["us", "them"])?;
    if args.name.trim().is_empty() {
        return Err(AppError::InvalidInput("signer name cannot be blank".into()));
    }
    let mut connection = db::open()?;
    let conn = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let existing = db::contract_get_or_404(&conn, &args.number)?;
    if matches!(existing.status.as_str(), "expired" | "terminated") {
        return Err(AppError::InvalidInput(
            "cannot record signatures on an expired or terminated contract".into(),
        ));
    }
    let already = match side.as_str() {
        "us" => existing.signed_by_us_name.is_some(),
        _ => existing.signed_by_them_name.is_some(),
    };
    if already && !args.force {
        return Err(AppError::InvalidInput(format!(
            "'{}' already has a recorded {} signature. Pass --force to overwrite it.",
            args.number, side
        )));
    }
    let date_iso = match args.date {
        Some(s) => parse_date(&s)?,
        None => chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };
    let c = db::contract_record_signature(
        &conn,
        &args.number,
        &side,
        &args.name,
        args.title.as_deref(),
        &date_iso,
    )?;
    conn.commit()?;
    print_success(ctx, &c, |c| {
        println!(
            "recorded {} signature on '{}'. Status: {}.",
            side, c.number, c.status
        );
    });
    Ok(())
}

// ─── duplicate / delete ───────────────────────────────────────────────────

fn cmd_duplicate(
    number: &str,
    client: Option<String>,
    r#as: Option<String>,
    ctx: Ctx,
) -> Result<()> {
    let mut conn = db::open()?;
    let src = db::contract_get_or_404(&conn, number)?;
    let src_clauses = db::clauses_for(&conn, src.id)?;

    let issuer = match r#as {
        Some(slug) => db::issuer_by_slug(&conn, &slug)?,
        None => db::issuer_list(&conn)?
            .into_iter()
            .find(|i| i.id == src.issuer_id)
            .ok_or_else(|| AppError::NotFound(format!("issuer #{}", src.issuer_id)))?,
    };
    let source_issuer = db::issuer_list(&conn)?
        .into_iter()
        .find(|i| i.id == src.issuer_id)
        .ok_or_else(|| AppError::NotFound("source issuer".into()))?;
    let source_client = db::client_list(&conn)?
        .into_iter()
        .find(|i| i.id == src.client_id)
        .ok_or_else(|| AppError::NotFound("source client".into()))?;
    let client_id = match client {
        Some(slug) => db::client_by_slug(&conn, &slug)?.id,
        None => src.client_id,
    };
    let year = chrono::Local::now().year();
    let new_number = db::next_contract_number(&conn, &issuer, year, &src.kind)?;
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let new_client = db::client_list(&conn)?
        .into_iter()
        .find(|c| c.id == client_id)
        .ok_or_else(|| AppError::NotFound("client".into()))?;
    let copied_title =
        if src.title == default_title(&src.kind, &source_issuer.name, &source_client.name) {
            default_title(&src.kind, &issuer.name, &new_client.name)
        } else {
            src.title.clone()
        };
    let new_contract = Contract {
        id: 0,
        number: new_number.clone(),
        kind: src.kind.clone(),
        issuer_id: issuer.id,
        client_id,
        title: copied_title,
        effective_date: today,
        // A copied absolute end date would predate the new effective date;
        // keep relative terms, drop absolute ones for the user to re-set.
        end_date: None,
        term_months: src.term_months,
        governing_law: src.governing_law.clone(),
        venue: src.venue.clone(),
        status: "draft".into(),
        sent_at: None,
        signed_at: None,
        terminated_at: None,
        notes: src.notes.clone(),
        fee_type: src.fee_type.clone(),
        fee_amount_minor: src.fee_amount_minor,
        fee_currency: src.fee_currency.clone(),
        fee_schedule: src.fee_schedule.clone(),
        terms_json: src.terms_json.clone(),
        clause_pack: src.clause_pack.clone(),
        clause_pack_version: src.clause_pack_version.clone(),
        default_template: src.default_template.clone(),
        signed_by_us_name: None,
        signed_by_us_title: None,
        signed_by_us_at: None,
        signed_by_them_name: None,
        signed_by_them_title: None,
        signed_by_them_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let fresh_clauses: Vec<ContractClauseRow> = src_clauses
        .into_iter()
        .map(|c| ContractClauseRow {
            id: 0,
            contract_id: 0,
            position: c.position,
            slug: c.slug,
            heading: c.heading,
            body: c.body,
        })
        .collect();
    db::contract_create(&mut conn, &new_contract, &fresh_clauses)?;
    let saved = db::contract_get(&conn, &new_number)?;
    print_success(ctx, &saved, |c| {
        println!("duplicated '{}' → '{}'", number, c.number);
    });
    Ok(())
}

fn cmd_delete(number: &str, force: bool, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    db::contract_delete(&conn, number, force)?;
    print_success(ctx, &number, |n| println!("deleted contract '{n}'"));
    Ok(())
}
