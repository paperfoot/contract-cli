use std::process::Command;

use crate::config;
use crate::error::{AppError, Result};
use crate::output::{print_success, Ctx};
use crate::typst_assets;

#[derive(serde::Serialize)]
pub struct DoctorReport {
    checks: Vec<Check>,
    summary: Summary,
}

#[derive(serde::Serialize)]
pub struct Check {
    name: String,
    status: &'static str,
    message: String,
}

#[derive(serde::Serialize)]
pub struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
}

pub fn run(ctx: Ctx) -> Result<()> {
    let mut checks = Vec::new();

    match Command::new("typst").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            checks.push(Check {
                name: "typst".into(),
                status: "pass",
                message: v,
            });
        }
        _ => checks.push(Check {
            name: "typst".into(),
            status: "fail",
            message: "typst not on PATH. Install: brew install typst".into(),
        }),
    }

    let cfg_path = config::config_path()?;
    checks.push(Check {
        name: "config".into(),
        status: "pass",
        message: format!("{} (exists: {})", cfg_path.display(), cfg_path.exists()),
    });

    let state = config::state_path()?;
    checks.push(Check {
        name: "state-dir".into(),
        status: "pass",
        message: format!("{} (exists: {})", state.display(), state.exists()),
    });

    typst_assets::ensure_extracted()?;
    let templates = typst_assets::list_templates()?;
    checks.push(Check {
        name: "templates".into(),
        status: if templates.is_empty() { "fail" } else { "pass" },
        message: format!("{} available: {}", templates.len(), templates.join(", ")),
    });

    let packs = crate::clauses::list_packs();
    checks.push(Check {
        name: "packs".into(),
        status: if packs.is_empty() { "fail" } else { "pass" },
        message: format!(
            "{} packs: {}",
            packs.len(),
            packs
                .iter()
                .map(|(k, p)| format!("{k}/{p}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    });

    match crate::db::open() {
        Ok(conn) => {
            checks.push(Check {
                name: "database".into(),
                status: "pass",
                message: format!("{} ok", config::db_path()?.display()),
            });
            match crate::db::issuer_list(&conn) {
                Ok(list) if list.is_empty() => checks.push(Check {
                    name: "issuers".into(),
                    status: "warn",
                    message: "no issuers configured (shared with invoice-cli). First run: contract issuer add <slug> --name X --address ...".into(),
                }),
                Ok(list) => {
                    let slugs: Vec<&str> = list.iter().map(|i| i.slug.as_str()).collect();
                    checks.push(Check {
                        name: "issuers".into(),
                        status: "pass",
                        message: format!("{} shared with invoice-cli: {}", list.len(), slugs.join(", ")),
                    });
                }
                Err(e) => checks.push(Check {
                    name: "issuers".into(),
                    status: "fail",
                    message: format!("{e}"),
                }),
            }
            match crate::db::client_list(&conn) {
                Ok(list) if list.is_empty() => checks.push(Check {
                    name: "clients".into(),
                    status: "warn",
                    message: "no clients configured (shared with invoice-cli). Add via: contract clients add <slug> ...".into(),
                }),
                Ok(list) => {
                    let slugs: Vec<&str> = list.iter().map(|c| c.slug.as_str()).collect();
                    let with_legal = list.iter().filter(|c| c.legal_name.is_some()).count();
                    let msg = if with_legal == list.len() {
                        format!("{} shared with invoice-cli, all with legal-name: {}", list.len(), slugs.join(", "))
                    } else {
                        format!("{} shared with invoice-cli ({} missing legal-name — fix via `contract clients edit <slug> --legal-name X --company-no Y --jurisdiction Z`): {}", list.len(), list.len() - with_legal, slugs.join(", "))
                    };
                    checks.push(Check {
                        name: "clients".into(),
                        status: if with_legal == list.len() { "pass" } else { "warn" },
                        message: msg,
                    });
                }
                Err(e) => checks.push(Check {
                    name: "clients".into(),
                    status: "fail",
                    message: format!("{e}"),
                }),
            }
        }
        Err(e) => checks.push(Check {
            name: "database".into(),
            status: "fail",
            message: format!("{e}"),
        }),
    }

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == "pass").count(),
        warn: checks.iter().filter(|c| c.status == "warn").count(),
        fail: checks.iter().filter(|c| c.status == "fail").count(),
    };
    let has_fail = summary.fail > 0;
    let report = DoctorReport { checks, summary };
    print_success(ctx, &report, |r| {
        for c in &r.checks {
            let icon = match c.status {
                "pass" => "✓",
                "warn" => "!",
                "fail" => "✗",
                _ => "?",
            };
            eprintln!("  {} {:<14} {}", icon, c.name, c.message);
        }
        eprintln!(
            "\n{} passing, {} warnings, {} failing",
            r.summary.pass, r.summary.warn, r.summary.fail
        );
    });
    if has_fail {
        return Err(AppError::Config("doctor found issues".into()));
    }
    Ok(())
}
