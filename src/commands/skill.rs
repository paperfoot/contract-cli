use crate::cli::SkillCmd;
use crate::error::Result;
use crate::output::{print_success, Ctx};

const SKILL_MD: &str = r#"---
name: contract-cli
description: >
  Generate beautiful, plain-English contracts (NDA, consulting, MSA, SOW,
  service) as PDF. Stateful — shares issuers + clients with invoice-cli via
  the Paperfoot accounting SQLite store. Composable clause packs let the user
  / agent pick, drop, reorder, or rewrite clauses. Use when the user asks to
  draft, generate, render, or sign a contract or NDA.
---

## contract-cli

`contract` is a stateful CLI for drafting and rendering business contracts.

### Quick start

```
contract issuer add acme --name "Acme Studio" --jurisdiction sg \
    --address "1 Marina Bay\nSingapore 018989"
contract clients add meridian --name "Meridian & Co." \
    --legal-name "Meridian & Co. Pty Ltd" --company-no "ACN 600 123 456" \
    --jurisdiction "Victoria, Australia" \
    --address "401 Collins Street\nMelbourne VIC 3000\nAustralia"

# Mutual NDA, 3-year term
contract new --kind nda --as acme --client meridian \
    --purpose "evaluation of a joint product line" --term-years 3

# Consulting agreement, fixed fee, 3-month term, IP to client
contract new --kind consulting --as acme --client meridian \
    --purpose "design and build a customer dashboard" \
    --fee fixed:8400:SGD --fee-schedule on-completion --term-months 3 \
    --deliverable "Discovery" --deliverable "Designs" --deliverable "Implementation" \
    --ip-assignment client --governing-law Singapore

contract list
contract render CTR-acme-2026-0001 --open                   # DRAFT watermark by default
contract render CTR-acme-2026-0001 --final --open           # clean copy for signature
contract sign CTR-acme-2026-0001 --side us --name "B. Djordjevic" --title CEO
contract sign CTR-acme-2026-0001 --side them --name "Sophie Lin" --title "Head of Marketing"
# status auto-bumps to 'signed' once both sides have signed
```

### Composing clauses

Every contract is built from a clause pack (`pack show <kind>` lists them).
By default it includes the pack's `default_clauses` in order. Customise per
contract:

```
contract clauses list CTR-acme-2026-0001
contract clauses add CTR-acme-2026-0001 non_solicit --from-file ./extra.md --position 8
contract clauses edit CTR-acme-2026-0001 termination --from-file ./our-termination.md
contract clauses remove CTR-acme-2026-0001 warranties
contract clauses move CTR-acme-2026-0001 governing_law 12
contract clauses reset CTR-acme-2026-0001          # back to pack default
```

A clause's `--body` / `--from-file` may use plain Markdown with paragraphs,
`- ` bullets, and `1. ` numbered lists. Pack clauses use `{{vars}}` like
`{{our_legal_name}}`, `{{their_legal_name}}`, `{{effective_date}}`,
`{{term_text}}`, `{{governing_law}}`, `{{jurisdiction_phrase}}`,
`{{fee_text}}`, `{{deliverables_block}}`, `{{ip_assignment_text}}`,
`{{confidentiality_years}}`, `{{termination_notice_days}}`, `{{purpose}}`.

### Tips

- Run `contract agent-info` for the full JSON manifest.
- Run `contract doctor --json` to verify typst + DB + packs + templates.
- Template chain at render: `--template` > `contract.default_template` > `"helvetica-nera"`.
- Templates: `helvetica-nera` (Swiss monochrome), `vienna-legal` (Bauhaus/terracotta), `editorial` (serif).
- Drafts render with a diagonal DRAFT watermark by default. Use `--final`
  for clean copies and `--draft` to force the watermark on a signed contract.
- These clauses are practical plain-English starting points, **not legal advice** —
  have a lawyer review for material engagements.
"#;

pub fn run(_cmd: SkillCmd, ctx: Ctx) -> Result<()> {
    let targets = [
        dirs_path(".claude/skills/contract-cli/SKILL.md"),
        dirs_path(".codex/skills/contract-cli/SKILL.md"),
        dirs_path(".gemini/skills/contract-cli/SKILL.md"),
    ];
    let mut written = Vec::new();
    for t in targets {
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

fn dirs_path(rel: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(rel)
}
