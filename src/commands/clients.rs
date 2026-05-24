use crate::cli::ClientCmd;
use crate::db::{self, Client};
use crate::error::Result;
use crate::output::{print_success, Ctx};

use super::split_multiline_arg;

pub fn run(cmd: ClientCmd, ctx: Ctx) -> Result<()> {
    let conn = db::open()?;
    match cmd {
        ClientCmd::Add {
            slug,
            name,
            legal_name,
            company_no,
            jurisdiction,
            attn,
            country,
            address,
            email,
            notes,
        } => {
            let c = Client {
                id: 0,
                slug,
                name,
                legal_name,
                company_no,
                legal_jurisdiction: jurisdiction,
                attn,
                country,
                tax_id: None,
                address: split_multiline_arg(&address),
                email,
                notes,
                default_issuer_slug: None,
                default_template: None,
            };
            let id = db::client_create(&conn, &c)?;
            let mut saved = c;
            saved.id = id;
            print_success(ctx, &saved, |c| println!("added client '{}' (#{})", c.slug, c.id));
            Ok(())
        }
        ClientCmd::Edit {
            slug,
            name,
            legal_name,
            company_no,
            jurisdiction,
            attn,
            country,
            address,
            email,
            notes,
        } => {
            let mut existing = db::client_by_slug(&conn, &slug)?;
            if let Some(v) = name { existing.name = v; }
            if let Some(v) = legal_name { existing.legal_name = Some(v); }
            if let Some(v) = company_no { existing.company_no = Some(v); }
            if let Some(v) = jurisdiction { existing.legal_jurisdiction = Some(v); }
            if let Some(v) = attn { existing.attn = Some(v); }
            if let Some(v) = country { existing.country = Some(v); }
            if let Some(v) = address { existing.address = split_multiline_arg(&v); }
            if let Some(v) = email { existing.email = Some(v); }
            if let Some(v) = notes { existing.notes = Some(v); }
            db::client_update(&conn, &existing)?;
            print_success(ctx, &existing, |c| println!("updated client '{}'", c.slug));
            Ok(())
        }
        ClientCmd::List => {
            let list = db::client_list(&conn)?;
            print_success(ctx, &list, |rows| {
                if rows.is_empty() {
                    println!("(no clients — add one with: contract clients add <slug> --name X --address ...)");
                } else {
                    for c in rows {
                        println!(
                            "  {:<14} {:<28} {}",
                            c.slug,
                            c.name,
                            c.legal_name.clone().unwrap_or_default()
                        );
                    }
                }
            });
            Ok(())
        }
        ClientCmd::Show { slug } => {
            let c = db::client_by_slug(&conn, &slug)?;
            print_success(ctx, &c, |c| println!("{:#?}", c));
            Ok(())
        }
        ClientCmd::Delete { slug } => {
            db::client_delete(&conn, &slug)?;
            print_success(ctx, &slug, |s| println!("deleted client '{s}'"));
            Ok(())
        }
    }
}
