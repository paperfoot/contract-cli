use crate::error::Result;
use crate::output::{print_raw, Ctx};

pub fn run(_ctx: Ctx) -> Result<()> {
    let config_path = crate::config::config_path()?.display().to_string();
    let state_dir = crate::config::state_path()?.display().to_string();
    let database = crate::config::db_path()?.display().to_string();

    let commands: &[(&str, &str)] = &[
        ("issuer add <slug> --name X --jurisdiction sg|uk|us|eu --address ...", "Register an issuer (your side of the contract). Shared with invoice-cli."),
        ("issuer edit <slug> [--name --legal-name --logo --output-dir ...]", "Update issuer fields."),
        ("issuer list | show <slug> | delete <slug>", "Manage issuers."),
        ("clients add <slug> --name X --address ... [--legal-name --company-no --jurisdiction]", "Register a counterparty. Legal fields are used by contract party blocks."),
        ("clients edit <slug> [...]", "Update client fields."),
        ("clients list | show <slug> | delete <slug>", "Manage clients."),
        ("new --kind nda|consulting|msa|sow|service --as <issuer> --client <client> [options...]", "Create a new contract from a clause pack."),
        ("list [--kind X --status Y --as Z]", "List contracts."),
        ("show <number>", "Show one contract + its clause list."),
        ("contracts edit <number> [--title --effective --end --term-months ...]", "Edit DRAFT contract metadata (sent/signed are immutable)."),
        ("render <number> [--template T --out PATH --open --draft | --final]", "Render to PDF. DRAFT watermark is implicit unless status is signed/active or --final is passed."),
        ("mark <number> draft|sent|signed|active|expired|terminated", "Update status; auto-stamps sent_at / signed_at / terminated_at."),
        ("sign <number> --side us|them --name 'Full Name' [--title T --date YYYY-MM-DD]", "Record one party's signature. When both sides sign, status auto-bumps to 'signed'."),
        ("contracts clauses list <number>", "Show clauses currently attached to a contract."),
        ("contracts clauses add <number> <slug> [--heading --body --from-file --position]", "Add a clause from the pack, or a custom one via --body / --from-file."),
        ("contracts clauses edit <number> <slug> [--heading --body --from-file]", "Override heading / body of an attached clause."),
        ("contracts clauses remove <number> <slug>", "Remove a clause."),
        ("contracts clauses move <number> <slug> <position>", "Re-order a clause."),
        ("contracts clauses reset <number>", "Reset to pack default."),
        ("pack list", "List available kind/pack combinations."),
        ("pack show <kind> [--pack standard]", "Show available clauses for a kind+pack."),
        ("template list | preview <name> [--kind nda|consulting]", "List or preview templates."),
        ("contracts duplicate <number> [--client C --as I]", "Clone a contract as a fresh draft."),
        ("contracts delete <number> [--force]", "Delete a contract. --force allows non-draft."),
        ("config show | path | set <key> <value>", "View / edit shared accounting config."),
        ("agent-info | info", "This manifest."),
        ("doctor", "Diagnose dependencies and DB."),
        ("skill install", "Install embedded Claude/Codex/Gemini skill."),
        ("update [--check]", "Self-update."),
    ];
    let mut commands_map = serde_json::Map::new();
    for (k, v) in commands {
        commands_map.insert(
            (*k).into(),
            serde_json::Value::String((*v).into()),
        );
    }
    let templates = crate::typst_assets::list_templates().unwrap_or_default();
    let packs = crate::clauses::list_packs();

    let manifest = serde_json::json!({
        "name": "contract",
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "commands": commands_map,
        "flags": {
            "--json": "Force JSON envelope output (auto when piped)",
            "--quiet": "Suppress human output",
        },
        "exit_codes": {
            "0": "Success",
            "1": "Transient error (IO, render)",
            "2": "Config error",
            "3": "Bad input / not found / ambiguous",
        },
        "envelope_schema": {
            "version": "1",
            "status": "success | error",
            "data": "… (success)",
            "error": "{ code, message, suggestion } (error)",
        },
        "config_path": config_path,
        "state_dir": state_dir,
        "database": database,
        "templates": templates,
        "packs": packs.into_iter().map(|(k, p)| serde_json::json!({"kind": k, "pack": p})).collect::<Vec<_>>(),
        "kinds": ["consulting", "nda", "msa", "sow", "service"],
        "fee_spec": "type:amount:currency — e.g. fixed:8400:SGD | hourly:200:SGD | daily:1500:SGD | retainer:5000:SGD",
        "first_run": [
            "contract doctor --json",
            "contract issuer list --json     # CHECK FIRST — shared with invoice-cli",
            "contract clients list --json    # CHECK FIRST — shared with invoice-cli",
            "# Only if a needed issuer/client is missing, then:",
            "contract issuer add <slug> --name <display-name> --jurisdiction sg|uk|us|eu --address \"line1\\nline2\"",
            "contract clients add <slug> --name <client-name> --legal-name <legal> --address \"line1\\nline2\"",
            "contract new --kind nda --as <issuer> --client <client> --purpose 'description'",
            "contract render <number> --open",
        ],
        "shared_state": {
            "database": database.clone(),
            "shared_with": ["invoice-cli (binary: invoice)"],
            "shared_tables": ["issuers", "clients", "number_series"],
            "discovery_workflow": "Before creating a new issuer or client, ALWAYS run `contract issuer list --json` and `contract clients list --json` (or the invoice-cli equivalents). The accounting suite's whole point is one source of truth — duplicating entities here pollutes invoicing too. If the entity exists under a different slug, prefer using the existing slug over creating a new one.",
            "legal_fields_on_clients": "Three columns added in V7 are contract-specific: `legal_name`, `company_no`, `legal_jurisdiction`. If an existing client (from invoice-cli) is missing them, fill via: contract clients edit <slug> --legal-name X --company-no Y --jurisdiction Z."
        },
        "examples": [
            { "goal": "Quick mutual NDA",
              "command": "contract new --kind nda --as acme --client meridian --purpose 'evaluation of a joint product' --term-years 3" },
            { "goal": "Consulting agreement with fixed fee",
              "command": "contract new --kind consulting --as acme --client meridian --purpose 'design a dashboard' --fee fixed:8400:SGD --term-months 3 --deliverable 'Design' --deliverable 'Build'" },
            { "goal": "Render with no watermark",
              "command": "contract render NDA-acme-2026-0001 --final --open" },
            { "goal": "Record both signatures",
              "command": "contract sign NDA-acme-2026-0001 --side us --name 'B. Djordjevic' --title CEO; contract sign NDA-acme-2026-0001 --side them --name 'Sophie Lin' --title 'Head of Marketing'" },
        ],
        "guardrails": [
            "BEFORE creating any issuer or client, run `contract issuer list` and `contract clients list` — the DB is shared with invoice-cli; duplicates pollute both tools.",
            "Run doctor before first use.",
            "Use --json for agents; stdout is data, stderr is diagnostics.",
            "Drafts render with a DRAFT watermark by default. Use --final to suppress before sending out for signature.",
            "Once a contract is sent/signed, only the status / signature columns can change.",
            "These clauses are practical plain-English starting points, not legal advice. Have a lawyer review for material engagements.",
        ],
    });
    print_raw(&manifest);
    Ok(())
}
