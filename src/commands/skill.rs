use crate::cli::SkillCmd;
use crate::error::Result;
use crate::output::{print_success, Ctx};

// The skill is a signpost, not a manual: the binary carries all workflow
// knowledge in agent-info and --help, so the skill body stays tiny and the
// description does the routing work.
const SKILL_MD: &str = r#"---
name: contract-cli
description: >
  Draft, render, and sign business contracts as beautiful PDFs — NDA, NCNDA
  (non-circumvention), consulting, MSA, SOW, service, loan. Use when the user
  asks to draft, generate, render, e-sign, or manage a contract, NDA, NCNDA,
  or loan agreement. Pick a PDF template by look with `contract template find
  "<description>"` and a contract kind with `contract kinds find "<need>"`.
  Shares issuers/clients with invoice-cli. Run `contract agent-info` for all
  commands, flags, and exit codes.
---

`contract` is a stateful CLI for drafting and rendering business contracts.

- Run `contract agent-info | jq` for the full manifest (commands, exit codes, envelope).
- The SQLite DB is SHARED with invoice-cli: ALWAYS `contract issuer list` and
  `contract clients list` before creating entities — reuse existing slugs.
- `contract kinds find "<what you need>"` and `contract template find "<look>"`
  resolve fuzzy descriptions to exact names; you make the final pick.
- Drafts render with a DRAFT watermark; `--final` produces the signing copy.
- Not legal advice — suggest lawyer review for material engagements.
"#;

fn targets() -> [std::path::PathBuf; 3] {
    [
        dirs_path(".claude/skills/contract-cli/SKILL.md"),
        dirs_path(".codex/skills/contract-cli/SKILL.md"),
        dirs_path(".gemini/skills/contract-cli/SKILL.md"),
    ]
}

pub fn run(cmd: SkillCmd, ctx: Ctx) -> Result<()> {
    match cmd {
        SkillCmd::Install => {
            let mut written = Vec::new();
            for t in targets() {
                if let Some(parent) = t.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&t, SKILL_MD)?;
                written.push(t.display().to_string());
            }
            print_success(ctx, &written, |paths| {
                for p in paths {
                    println!("installed → {}", p);
                }
            });
            Ok(())
        }
        SkillCmd::Status => {
            #[derive(serde::Serialize)]
            struct SkillStatus {
                path: String,
                installed: bool,
                current: bool,
            }
            let statuses: Vec<SkillStatus> = targets()
                .iter()
                .map(|t| {
                    let on_disk = std::fs::read_to_string(t).ok();
                    SkillStatus {
                        path: t.display().to_string(),
                        installed: on_disk.is_some(),
                        current: on_disk.as_deref() == Some(SKILL_MD),
                    }
                })
                .collect();
            print_success(ctx, &statuses, |ss| {
                for s in ss {
                    let state = if s.current {
                        "current"
                    } else if s.installed {
                        "STALE — run: contract skill install"
                    } else {
                        "not installed"
                    };
                    println!("  {} — {}", s.path, state);
                }
            });
            Ok(())
        }
    }
}

fn dirs_path(rel: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(rel)
}
