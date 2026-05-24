use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use chrono::{Datelike, NaiveDate, Utc};
use serde_json::{json, Value};

use crate::cli::{ContractCmd, ContractListArgs, ContractNewArgs, ContractRenderArgs, SignArgs};
use crate::clauses;
use crate::db::{self, Contract, ContractClauseRow};
use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};
use crate::render;

pub fn run(cmd: ContractCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        ContractCmd::New(args) => cmd_new(args, ctx),
        ContractCmd::List(args) => cmd_list(args, ctx),
        ContractCmd::Show { number } => cmd_show(&number, ctx),
        ContractCmd::Edit {
            number,
            client,
            title,
            effective,
            end,
            term_months,
            governing_law,
            venue,
            fee,
            fee_schedule,
            notes,
            template,
        } => cmd_edit(
            number,
            client,
            title,
            effective,
            end,
            term_months,
            governing_law,
            venue,
            fee,
            fee_schedule,
            notes,
            template,
            ctx,
        ),
        ContractCmd::Render(args) => cmd_render(args, ctx),
        ContractCmd::Mark { number, status } => cmd_mark(&number, &status, ctx),
        ContractCmd::Sign(args) => cmd_sign(args, ctx),
        ContractCmd::Clauses(cmd) => super::clauses::run(cmd, ctx),
        ContractCmd::Duplicate { number, client, r#as } => cmd_duplicate(&number, client, r#as, ctx),
        ContractCmd::Delete { number, force } => cmd_delete(&number, force, ctx),
    }
}

// ─── new ─────────────────────────────────────────────────────────────────

const KINDS: &[&str] = &["consulting", "nda", "msa", "sow", "service"];

fn cmd_new(args: ContractNewArgs, ctx: Ctx) -> Result<()> {
    if !KINDS.contains(&args.kind.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "unknown kind '{}'. Expected one of: {}",
            args.kind,
            KINDS.join(", ")
        )));
    }
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

    let effective_iso = match args.effective {
        Some(s) => parse_date(&s)?,
        None => Utc::now().date_naive().format("%Y-%m-%d").to_string(),
    };
    let end_iso = match args.end {
        Some(s) => Some(parse_date(&s)?),
        None => None,
    };
    let term_months = match (args.term_months, args.term_years) {
        (Some(_), Some(_)) => {
            return Err(AppError::InvalidInput(
                "pass at most one of --term-months / --term-years".into(),
            ))
        }
        (Some(m), None) => Some(m),
        (None, Some(y)) => Some(y * 12),
        (None, None) => None,
    };
    if end_iso.is_some() && term_months.is_some() {
        return Err(AppError::InvalidInput(
            "--end and --term-months/--term-years are mutually exclusive".into(),
        ));
    }

    let governing_law = args
        .governing_law
        .unwrap_or_else(|| issuer.jurisdiction.profile().country.to_string());
    let venue = args.venue;

    let title = args
        .title
        .unwrap_or_else(|| default_title(&args.kind, &issuer.name, &client.name));

    // Parse fee
    let (fee_type, fee_amount_minor, fee_currency) = match args.fee.as_deref() {
        None => (None, None, None),
        Some(spec) => {
            let (t, a, c) = parse_fee(spec)?;
            (Some(t), Some(a), Some(c))
        }
    };
    let fee_schedule = args.fee_schedule;

    // Build terms_json
    let mut terms_obj = serde_json::Map::new();
    if let Some(p) = &args.purpose {
        terms_obj.insert("purpose".into(), Value::String(p.clone()));
    }
    if let Some(n) = args.termination_notice_days {
        terms_obj.insert("termination_notice_days".into(), json!(n));
    }
    if args.kind == "nda" {
        terms_obj.insert(
            "mutuality".into(),
            Value::String(args.mutuality.clone().unwrap_or_else(|| "mutual".into())),
        );
        terms_obj.insert(
            "disclosing_side".into(),
            Value::String(args.disclosing_side.clone().unwrap_or_else(|| {
                if args.mutuality.as_deref() == Some("unilateral") {
                    "us".into()
                } else {
                    "both".into()
                }
            })),
        );
        terms_obj.insert("confidentiality_years".into(), json!(3));
    } else if matches!(args.kind.as_str(), "consulting" | "msa" | "sow" | "service") {
        if !args.deliverables.is_empty() {
            terms_obj.insert(
                "deliverables".into(),
                Value::Array(args.deliverables.iter().cloned().map(Value::String).collect()),
            );
        }
        if let Some(ip) = &args.ip_assignment {
            terms_obj.insert("ip_assignment".into(), Value::String(ip.clone()));
        }
        terms_obj
            .entry("confidentiality_years".to_string())
            .or_insert(json!(3));
    }
    let terms_json = Value::Object(terms_obj).to_string();

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
            heading: None,
            body: None,
        })
        .collect();

    // Generate number
    let year = NaiveDate::parse_from_str(&effective_iso, "%Y-%m-%d")
        .map(|d| d.year())
        .unwrap_or_else(|_| Utc::now().year());
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
            c.kind, c.number, c.title, clause_rows.len()
        );
    });
    Ok(())
}

fn default_title(kind: &str, issuer_name: &str, client_name: &str) -> String {
    match kind {
        "nda" => format!("NDA — {issuer_name} & {client_name}"),
        "consulting" => format!("Consulting Agreement — {issuer_name} × {client_name}"),
        "msa" => format!("Master Services Agreement — {issuer_name} & {client_name}"),
        "sow" => format!("Statement of Work — {issuer_name} × {client_name}"),
        "service" => format!("Service Agreement — {issuer_name} for {client_name}"),
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
    if parts.len() < 3 {
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
    let amount_major: f64 = parts[1].parse().map_err(|_| {
        AppError::InvalidInput(format!("invalid fee amount '{}': must be a number", parts[1]))
    })?;
    let currency = parts[2].to_uppercase();
    // allow trailing "/month" etc — ignore for now
    let amount_minor = (amount_major * 100.0).round() as i64;
    Ok((kind, amount_minor, currency))
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
    let view = ContractView { contract: &c, clauses: clauses.clone() };
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
                v.contract.fee_amount_minor.map(|m| (m as f64) / 100.0).unwrap_or(0.0),
                v.contract.fee_currency.clone().unwrap_or_default()
            );
        }
        println!("  Pack:           {} v{}", v.contract.clause_pack, v.contract.clause_pack_version);
        println!("  Clauses ({}):", v.clauses.len());
        for cl in &v.clauses {
            println!("    {:>2}. {}", cl.position + 1, cl.slug);
        }
    });
    Ok(())
}

// ─── edit ─────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn cmd_edit(
    number: String,
    client: Option<String>,
    title: Option<String>,
    effective: Option<String>,
    end: Option<String>,
    term_months: Option<i64>,
    governing_law: Option<String>,
    venue: Option<String>,
    fee: Option<String>,
    fee_schedule: Option<String>,
    notes: Option<String>,
    template: Option<String>,
    ctx: Ctx,
) -> Result<()> {
    let conn = db::open()?;
    let mut c = db::contract_get_or_404(&conn, &number)?;
    if let Some(slug) = client {
        c.client_id = db::client_by_slug(&conn, &slug)?.id;
    }
    if let Some(v) = title { c.title = v; }
    if let Some(v) = effective { c.effective_date = parse_date(&v)?; }
    if let Some(v) = end {
        c.end_date = Some(parse_date(&v)?);
        c.term_months = None;
    }
    if let Some(v) = term_months {
        c.term_months = Some(v);
        c.end_date = None;
    }
    if let Some(v) = governing_law { c.governing_law = v; }
    if let Some(v) = venue { c.venue = Some(v); }
    if let Some(v) = fee {
        let (t, a, cur) = parse_fee(&v)?;
        c.fee_type = Some(t);
        c.fee_amount_minor = Some(a);
        c.fee_currency = Some(cur);
    }
    if let Some(v) = fee_schedule { c.fee_schedule = Some(v); }
    if let Some(v) = notes { c.notes = Some(v); }
    if let Some(v) = template { c.default_template = Some(v); }
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
    let pack = clauses::load_pack(&c.kind, &c.clause_pack)?;

    let template = args
        .template
        .or_else(|| c.default_template.clone())
        .unwrap_or_else(|| "helvetica-nera".to_string());

    let out_path: PathBuf = match args.out {
        Some(p) => PathBuf::from(p),
        None => {
            let dir = issuer
                .default_output_dir
                .clone()
                .map(|s| PathBuf::from(render::expand_tilde(&s)))
                .unwrap_or_else(render::default_output_dir);
            std::fs::create_dir_all(&dir)?;
            dir.join(format!("{}.pdf", c.number))
        }
    };

    let mut data = render::build_render_data(
        &c, &issuer, &client, &clause_rows, &pack, args.draft, args.final_render,
    )?;
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

fn cmd_mark(number: &str, status: &str, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    db::contract_set_status(&conn, number, status)?;
    let c = db::contract_get(&conn, number)?;
    print_success(ctx, &c, |c| println!("'{}' → {}", c.number, c.status));
    Ok(())
}

fn cmd_sign(args: SignArgs, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    let date_iso = match args.date {
        Some(s) => parse_date(&s)?,
        None => Utc::now().date_naive().format("%Y-%m-%d").to_string(),
    };
    let c = db::contract_record_signature(
        &conn,
        &args.number,
        &args.side,
        &args.name,
        args.title.as_deref(),
        &date_iso,
    )?;
    print_success(ctx, &c, |c| {
        println!(
            "recorded {} signature on '{}'. Status: {}.",
            args.side, c.number, c.status
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
    let client_id = match client {
        Some(slug) => db::client_by_slug(&conn, &slug)?.id,
        None => src.client_id,
    };
    let year = Utc::now().year();
    let new_number = db::next_contract_number(&conn, &issuer, year, &src.kind)?;
    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let new_contract = Contract {
        id: 0,
        number: new_number.clone(),
        kind: src.kind.clone(),
        issuer_id: issuer.id,
        client_id,
        title: src.title.clone(),
        effective_date: today,
        end_date: src.end_date.clone(),
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
