use crate::cli::PackCmd;
use crate::clauses;
use crate::error::Result;
use crate::output::{print_success, Ctx};

pub fn run(cmd: PackCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        PackCmd::List => {
            let mut packs = clauses::list_packs();
            packs.sort();
            print_success(ctx, &packs, |ps| {
                if ps.is_empty() {
                    println!("(no packs)");
                } else {
                    println!("  Kind        Pack");
                    for (k, p) in ps {
                        println!("  {:<12} {}", k, p);
                    }
                }
            });
            Ok(())
        }
        PackCmd::Show { kind, pack } => {
            let p = clauses::load_pack(&kind, &pack)?;
            #[derive(serde::Serialize)]
            struct View {
                kind: String,
                pack: String,
                name: String,
                version: String,
                default_clauses: Vec<String>,
                all_clauses: Vec<(String, String)>,
            }
            let mut all_clauses: Vec<(String, String)> = p
                .clauses
                .iter()
                .map(|(slug, def)| (slug.clone(), def.heading.clone()))
                .collect();
            all_clauses.sort();
            let view = View {
                kind: p.pack.kind.clone(),
                pack: p.pack.slug.clone(),
                name: p.pack.name.clone(),
                version: p.pack.version.clone(),
                default_clauses: p.pack.default_clauses.clone(),
                all_clauses,
            };
            print_success(ctx, &view, |v| {
                println!("{} — {} (v{})", v.kind, v.name, v.version);
                println!("Default order:");
                for (i, s) in v.default_clauses.iter().enumerate() {
                    println!("  {:>2}. {}", i + 1, s);
                }
                println!("\nAll available clauses:");
                for (slug, heading) in &v.all_clauses {
                    println!("  {:<22} {}", slug, heading);
                }
            });
            Ok(())
        }
    }
}
