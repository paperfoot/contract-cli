use crate::cli::KindsCmd;
use crate::error::{AppError, Result};
use crate::kinds;
use crate::output::{print_success, Ctx};

#[derive(serde::Serialize)]
struct KindInfo {
    kind: &'static str,
    prefix: &'static str,
    roles: (&'static str, &'static str),
    description: &'static str,
    tags: &'static [&'static str],
}

fn info(k: &'static kinds::KindSpec) -> KindInfo {
    KindInfo {
        kind: k.kind,
        prefix: k.prefix,
        roles: k.roles,
        description: k.description,
        tags: k.tags,
    }
}

pub fn run(cmd: KindsCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        KindsCmd::List => {
            let all: Vec<KindInfo> = kinds::KINDS.iter().map(info).collect();
            print_success(ctx, &all, |ks| {
                for k in ks {
                    println!("  {:<12} {}", k.kind, k.description);
                }
            });
            Ok(())
        }
        KindsCmd::Find { query } => {
            let query = query.join(" ");
            let mut scored: Vec<(f64, &kinds::KindSpec)> = kinds::KINDS
                .iter()
                .map(|k| (kinds::score(&query, k.description, k.tags), k))
                .filter(|(s, _)| *s > 0.0)
                .collect();
            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            if scored.is_empty() {
                return Err(AppError::NotFound(format!(
                    "no contract kind matches '{query}'. Run: contract kinds list"
                )));
            }
            #[derive(serde::Serialize)]
            struct Match {
                kind: &'static str,
                score: f64,
                description: &'static str,
            }
            let matches: Vec<Match> = scored
                .iter()
                .map(|(s, k)| Match {
                    kind: k.kind,
                    score: (*s * 100.0).round() / 100.0,
                    description: k.description,
                })
                .collect();
            print_success(ctx, &matches, |ms| {
                for m in ms {
                    println!("  {:<12} {:>5.2}  {}", m.kind, m.score, m.description);
                }
            });
            Ok(())
        }
    }
}
