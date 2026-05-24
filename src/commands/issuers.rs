use crate::cli::IssuerCmd;
use crate::db::{self, Issuer};
use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};
use crate::tax::Jurisdiction;

use super::split_multiline_arg;

pub fn run(cmd: IssuerCmd, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    match cmd {
        IssuerCmd::Add {
            slug,
            name,
            legal_name,
            jurisdiction,
            tax_id,
            company_no,
            address,
            email,
            phone,
            logo,
            output_dir,
        } => {
            let jur = Jurisdiction::from_str(&jurisdiction).ok_or_else(|| {
                AppError::InvalidInput(format!("unknown jurisdiction '{jurisdiction}'"))
            })?;
            let issuer = Issuer {
                id: 0,
                slug: slug.clone(),
                name,
                legal_name,
                jurisdiction: jur,
                tax_registered: false,
                tax_id,
                company_no,
                tagline: None,
                address: split_multiline_arg(&address),
                email,
                phone,
                bank_details: None,
                default_template: "helvetica-nera".into(),
                currency: None,
                symbol: None,
                number_format: "{issuer}-{year}-{seq:04}".into(),
                logo_path: logo,
                default_output_dir: output_dir,
                default_notes: None,
            };
            let id = db::issuer_create(&conn, &issuer)?;
            let mut saved = issuer;
            saved.id = id;
            print_success(ctx, &saved, |i| {
                println!("added issuer '{}' (#{})", i.slug, i.id)
            });
            Ok(())
        }
        IssuerCmd::Edit {
            slug,
            name,
            legal_name,
            jurisdiction,
            tax_id,
            company_no,
            address,
            email,
            phone,
            logo,
            logo_clear,
            output_dir,
        } => {
            let mut existing = db::issuer_by_slug(&conn, &slug)?;
            if let Some(v) = name { existing.name = v; }
            if let Some(v) = legal_name { existing.legal_name = Some(v); }
            if let Some(v) = jurisdiction {
                existing.jurisdiction = Jurisdiction::from_str(&v).ok_or_else(|| {
                    AppError::InvalidInput(format!("unknown jurisdiction '{v}'"))
                })?;
            }
            if let Some(v) = tax_id { existing.tax_id = Some(v); }
            if let Some(v) = company_no { existing.company_no = Some(v); }
            if let Some(v) = address { existing.address = split_multiline_arg(&v); }
            if let Some(v) = email { existing.email = Some(v); }
            if let Some(v) = phone { existing.phone = Some(v); }
            if logo_clear { existing.logo_path = None; }
            if let Some(v) = logo { existing.logo_path = Some(v); }
            if let Some(v) = output_dir { existing.default_output_dir = Some(v); }
            db::issuer_update(&conn, &existing)?;
            print_success(ctx, &existing, |i| println!("updated issuer '{}'", i.slug));
            Ok(())
        }
        IssuerCmd::List => {
            let list = db::issuer_list(&conn)?;
            print_success(ctx, &list, |rows| {
                if rows.is_empty() {
                    println!("(no issuers — add one with: contract issuer add <slug> --name X --address ...)");
                } else {
                    for i in rows {
                        println!(
                            "  {:<14} {:<24} {}",
                            i.slug,
                            i.name,
                            i.legal_name.clone().unwrap_or_default()
                        );
                    }
                }
            });
            Ok(())
        }
        IssuerCmd::Show { slug } => {
            let i = db::issuer_by_slug(&conn, &slug)?;
            print_success(ctx, &i, |i| println!("{:#?}", i));
            Ok(())
        }
        IssuerCmd::Delete { slug } => {
            db::issuer_delete(&conn, &slug)?;
            print_success(ctx, &slug, |s| println!("deleted issuer '{s}'"));
            Ok(())
        }
    }
}
