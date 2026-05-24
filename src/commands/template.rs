use chrono::Utc;
use std::path::PathBuf;

use crate::cli::TemplateCmd;
use crate::clauses;
use crate::db::{Client, Contract, ContractClauseRow, Issuer};
use crate::error::Result;
use crate::output::{print_success, Ctx};
use crate::render;
use crate::tax::Jurisdiction;
use crate::typst_assets;

pub fn run(cmd: TemplateCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        TemplateCmd::List => {
            let names = typst_assets::list_templates()?;
            print_success(ctx, &names, |ns| {
                if ns.is_empty() {
                    println!("(no templates)");
                } else {
                    for n in ns {
                        println!("  {n}");
                    }
                }
            });
            Ok(())
        }
        TemplateCmd::Preview { name, kind, out } => {
            let issuer = sample_issuer();
            let client = sample_client();
            let contract = sample_contract(&kind);
            let pack = clauses::load_pack(&kind, &contract.clause_pack)?;
            let clause_rows: Vec<ContractClauseRow> = pack
                .pack
                .default_clauses
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
            let mut data =
                render::build_render_data(&contract, &issuer, &client, &clause_rows, &pack, false, true)?;
            let out_path =
                PathBuf::from(out.unwrap_or_else(|| format!("preview-{name}-{kind}.pdf")));
            render::render_to_pdf(&name, &mut data, &issuer, &out_path)?;
            print_success(ctx, &out_path.display().to_string(), |p| {
                println!("preview → {p}");
            });
            Ok(())
        }
    }
}

fn sample_issuer() -> Issuer {
    Issuer {
        id: 0,
        slug: "acme".into(),
        name: "Acme Studio".into(),
        legal_name: Some("Acme Studio Pte. Ltd.".into()),
        jurisdiction: Jurisdiction::Sg,
        tax_registered: false,
        tax_id: None,
        company_no: Some("202312345A".into()),
        tagline: None,
        address: vec![
            "1 Marina Bay".into(),
            "Singapore 018989".into(),
        ],
        email: Some("hello@acme.example".into()),
        phone: None,
        bank_details: None,
        default_template: "helvetica-nera".into(),
        currency: None,
        symbol: None,
        number_format: "{issuer}-{year}-{seq:04}".into(),
        logo_path: None,
        default_output_dir: None,
        default_notes: None,
    }
}

fn sample_client() -> Client {
    Client {
        id: 0,
        slug: "meridian".into(),
        name: "Meridian & Co.".into(),
        legal_name: Some("Meridian & Co. Pty Ltd".into()),
        company_no: Some("ACN 600 123 456".into()),
        legal_jurisdiction: Some("Victoria, Australia".into()),
        attn: Some("Sophie Lin, Head of Marketing".into()),
        country: Some("AU".into()),
        tax_id: None,
        address: vec![
            "401 Collins Street".into(),
            "Melbourne VIC 3000".into(),
            "Australia".into(),
        ],
        email: Some("sophie@meridian.example".into()),
        notes: None,
        default_issuer_slug: None,
        default_template: None,
    }
}

fn sample_contract(kind: &str) -> Contract {
    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let terms = match kind {
        "nda" => serde_json::json!({
            "mutuality": "mutual",
            "disclosing_side": "both",
            "purpose": "a potential collaboration on a longevity research project",
            "confidentiality_years": 3,
        }),
        "consulting" => serde_json::json!({
            "purpose": "the design and delivery of a customer-facing dashboard for the Client's flagship product",
            "deliverables": [
                "Discovery interviews and a one-page strategy memo",
                "Three rounds of high-fidelity Figma designs",
                "Production-ready front-end implementation in React",
                "One handover session with the Client's engineering team",
            ],
            "ip_assignment": "client",
            "confidentiality_years": 3,
            "termination_notice_days": 30,
        }),
        "msa" => serde_json::json!({
            "ip_assignment": "client",
            "confidentiality_years": 3,
            "termination_notice_days": 30,
        }),
        "sow" => serde_json::json!({
            "purpose": "Implementation of the customer dashboard described in the kick-off memo dated 2026-04-01.",
            "deliverables": [
                "Functional prototype by week 4",
                "Production deployment by week 10",
            ],
        }),
        "service" => serde_json::json!({
            "purpose": "managed hosting and monthly performance reviews for the Customer's web application",
            "deliverables": [
                "24/7 monitoring of production endpoints",
                "Monthly performance review and recommendations",
            ],
            "termination_notice_days": 60,
        }),
        _ => serde_json::json!({}),
    };
    let (fee_type, fee_minor, fee_cur, fee_sched) = match kind {
        "consulting" => (Some("fixed".into()), Some(84_000_00i64), Some("SGD".into()), Some("on-completion".into())),
        "msa" => (None, None, None, None),
        "sow" => (Some("fixed".into()), Some(34_000_00i64), Some("SGD".into()), Some("on-milestone".into())),
        "service" => (Some("retainer".into()), Some(5_000_00i64), Some("SGD".into()), Some("monthly".into())),
        _ => (None, None, None, None),
    };
    let title = match kind {
        "nda" => "Mutual NDA — Acme × Meridian".into(),
        "consulting" => "Customer Dashboard — Consulting Engagement".into(),
        "msa" => "Master Services Agreement — Acme × Meridian".into(),
        "sow" => "SOW #1 — Customer Dashboard Implementation".into(),
        "service" => "Managed Hosting & Performance Reviews".into(),
        _ => format!("Sample {kind} agreement"),
    };
    Contract {
        id: 0,
        number: format!("{}-acme-2026-0001", prefix_for(kind)),
        kind: kind.to_string(),
        issuer_id: 0,
        client_id: 0,
        title,
        effective_date: today,
        end_date: None,
        term_months: Some(12),
        governing_law: "Singapore".into(),
        venue: None,
        status: "draft".into(),
        sent_at: None,
        signed_at: None,
        terminated_at: None,
        notes: None,
        fee_type,
        fee_amount_minor: fee_minor,
        fee_currency: fee_cur,
        fee_schedule: fee_sched,
        terms_json: terms.to_string(),
        clause_pack: "standard".into(),
        clause_pack_version: "1.0".into(),
        default_template: None,
        signed_by_us_name: None,
        signed_by_us_title: None,
        signed_by_us_at: None,
        signed_by_them_name: None,
        signed_by_them_title: None,
        signed_by_them_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn prefix_for(kind: &str) -> &'static str {
    match kind {
        "consulting" => "CTR",
        "nda" => "NDA",
        "msa" => "MSA",
        "sow" => "SOW",
        "service" => "SVC",
        _ => "DOC",
    }
}
