// Canonical agent-info manifest per agent-cli-framework
// schemas/agent-info.schema.json: `commands` keys are routable paths, values
// are {description, aliases?, args, options}. Every key must satisfy
// `contract <key> --help` — conformance walks them verbatim.

use serde_json::json;

use crate::error::Result;
use crate::output::{print_raw, Ctx};

fn arg(name: &str, ty: &str, required: bool, desc: &str) -> serde_json::Value {
    json!({ "name": name, "kind": "positional", "type": ty, "required": required, "description": desc })
}

fn opt(name: &str, ty: &str, desc: &str) -> serde_json::Value {
    json!({ "name": name, "type": ty, "required": false, "description": desc })
}

fn req_opt(name: &str, ty: &str, desc: &str) -> serde_json::Value {
    json!({ "name": name, "type": ty, "required": true, "description": desc })
}

fn cmd(desc: &str, aliases: &[&str], args: Vec<serde_json::Value>, options: Vec<serde_json::Value>) -> serde_json::Value {
    let mut v = json!({ "description": desc, "args": args, "options": options });
    if !aliases.is_empty() {
        v["aliases"] = json!(aliases);
    }
    v
}

pub fn run(_ctx: Ctx) -> Result<()> {
    let config_path = crate::config::config_path()?.display().to_string();
    let state_dir = crate::config::state_path()?.display().to_string();
    let database = crate::config::db_path()?.display().to_string();

    let slug = |d: &str| vec![arg("slug", "string", true, d)];
    let number = || vec![arg("number", "string", true, "Contract number, e.g. NDA-acme-2026-0001")];

    let entity_opts = |contract_side: bool| {
        let mut o = vec![
            opt("--name", "string", "Display name"),
            opt("--legal-name", "string", "Legal entity name used on contracts"),
            opt("--company-no", "string", "Registration / company number"),
            opt("--jurisdiction", "string", "Jurisdiction (issuers: sg|uk|us|eu; clients: free text)"),
            opt("--address", "string", "Address lines separated by \\n"),
            opt("--email", "string", "Contact email"),
        ];
        if contract_side {
            o.push(opt("--tax-id", "string", "Tax / VAT id"));
            o.push(opt("--phone", "string", "Phone"));
            o.push(opt("--logo", "string", "Path to logo image for the contract header"));
            o.push(opt("--output-dir", "string", "Default output dir for render"));
        } else {
            o.push(opt("--attn", "string", "Attention line"));
            o.push(opt("--country", "string", "ISO country code"));
            o.push(opt("--notes", "string", "Free-form notes"));
        }
        o
    };

    let new_opts = vec![
        req_opt("--kind", "string", "nda | ncnda | consulting | msa | sow | service | loan"),
        opt("--as", "string", "Issuer slug (your side); falls back to client default, then config"),
        req_opt("--client", "string", "Client slug (counterparty)"),
        opt("--title", "string", "Contract title (defaults per kind)"),
        opt("--effective", "string", "Effective date YYYY-MM-DD (default today)"),
        opt("--end", "string", "End date YYYY-MM-DD (mutually exclusive with --term-months)"),
        opt("--term-months", "int", "Term length in months"),
        opt("--term-years", "int", "Term length in years (sugar for months × 12)"),
        opt("--governing-law", "string", "Governing law, e.g. 'England and Wales'"),
        opt("--venue", "string", "Court venue override"),
        opt("--fee", "string", "type:amount:currency — fixed:8400:SGD | hourly:200:SGD | daily:1500:SGD | retainer:5000:SGD"),
        opt("--fee-schedule", "string", "on-completion | monthly | on-milestone | upon-invoice"),
        opt("--mutuality", "string", "nda/ncnda: mutual | unilateral"),
        opt("--disclosing-side", "string", "nda: us | them | both"),
        opt("--purpose", "string", "Purpose / scope summary"),
        opt("--deliverable", "string", "Deliverable line (repeatable)"),
        opt("--ip-assignment", "string", "client | consultant | shared"),
        opt("--termination-notice-days", "int", "Notice days for termination for convenience"),
        opt("--term", "string", "Arbitrary term key=value for pack {{vars}} (repeatable), e.g. principal_text='£10,000 (ten thousand pounds)'"),
        opt("--pack", "string", "Clause pack slug (default: standard)"),
        opt("--include", "string", "Extra pack clause slug to include (repeatable)"),
        opt("--exclude", "string", "Default pack clause slug to drop (repeatable)"),
        opt("--template", "string", "Default render template for this contract"),
        opt("--notes", "string", "Internal notes (never rendered)"),
    ];

    let render_opts = vec![
        opt("--template", "string", "Template override (see: template list)"),
        opt("--out", "string", "Output PDF path"),
        opt("--open", "bool", "Open the PDF after rendering"),
        opt("--draft", "bool", "Force the DRAFT watermark"),
        opt("--final", "bool", "Force a clean copy (no watermark)"),
    ];

    let commands = json!({
        "issuer add": cmd("Register an issuer (your side; shared with invoice-cli)", &["issuer new"], slug("Issuer slug"), entity_opts(true)),
        "issuer edit": cmd("Update issuer fields", &[], slug("Issuer slug"), {
            let mut o = entity_opts(true);
            o.push(opt("--logo-clear", "bool", "Remove the stored logo"));
            o
        }),
        "issuer list": cmd("List issuers (shared with invoice-cli)", &["issuer ls"], vec![], vec![]),
        "issuer show": cmd("Show one issuer", &["issuer get"], slug("Issuer slug"), vec![]),
        "issuer delete": cmd("Delete an issuer", &["issuer rm"], slug("Issuer slug"), vec![]),
        "clients add": cmd("Register a counterparty (shared with invoice-cli)", &["client add", "clients new"], slug("Client slug"), entity_opts(false)),
        "clients edit": cmd("Update client fields (legal-name/company-no/jurisdiction feed the party block)", &["client edit"], slug("Client slug"), entity_opts(false)),
        "clients list": cmd("List clients (shared with invoice-cli)", &["clients ls"], vec![], vec![]),
        "clients show": cmd("Show one client", &["clients get"], slug("Client slug"), vec![]),
        "clients delete": cmd("Delete a client", &["clients rm"], slug("Client slug"), vec![]),
        "new": cmd("Create a contract from a clause pack", &["contracts new", "contracts create"], vec![], new_opts),
        "list": cmd("List contracts", &["ls", "contracts list"], vec![], vec![
            opt("--kind", "string", "Filter by kind"),
            opt("--status", "string", "Filter by status"),
            opt("--as", "string", "Filter by issuer slug"),
        ]),
        "show": cmd("Show one contract + clause list", &["get", "contracts show"], number(), vec![]),
        "edit": cmd("Edit DRAFT contract metadata (sent/signed are immutable)", &["contracts edit"], number(), vec![
            opt("--client", "string", "Re-point at a different client slug"),
            opt("--title", "string", "New title"),
            opt("--effective", "string", "Effective date YYYY-MM-DD"),
            opt("--end", "string", "End date (clears term-months)"),
            opt("--term-months", "int", "Term months (clears end date)"),
            opt("--governing-law", "string", "Governing law"),
            opt("--venue", "string", "Venue"),
            opt("--fee", "string", "type:amount:currency"),
            opt("--fee-schedule", "string", "on-completion | monthly | on-milestone | upon-invoice"),
            opt("--term", "string", "Arbitrary term key=value (repeatable)"),
            opt("--notes", "string", "Internal notes"),
            opt("--template", "string", "Default template"),
        ]),
        "render": cmd("Render to PDF (DRAFT watermark unless executed or --final)", &["contracts render"], number(), render_opts),
        "mark": cmd("Update status: draft|sent|signed|active|expired|terminated (forward-only once executed)", &["contracts mark"], vec![
            arg("number", "string", true, "Contract number"),
            arg("status", "string", true, "Target status"),
        ], vec![]),
        "sign": cmd("Record one party's signature; auto-bumps to signed when both sides sign", &["contracts sign"], number(), vec![
            req_opt("--side", "string", "us | them"),
            req_opt("--name", "string", "Signer full name"),
            opt("--title", "string", "Signer title"),
            opt("--date", "string", "Signature date YYYY-MM-DD (default today)"),
            opt("--force", "bool", "Overwrite an already-recorded signature"),
        ]),
        "contracts clauses list": cmd("Show clauses attached to a contract", &["contracts clauses ls"], number(), vec![]),
        "contracts clauses add": cmd("Add a pack clause, or a custom clause via --body/--from-file", &[], vec![
            arg("number", "string", true, "Contract number"),
            arg("slug", "string", true, "Clause slug"),
        ], vec![
            opt("--heading", "string", "Override heading"),
            opt("--body", "string", "Custom body markdown"),
            opt("--from-file", "string", "Custom body markdown from file"),
            opt("--position", "int", "Zero-indexed insert position (default: end)"),
        ]),
        "contracts clauses edit": cmd("Override heading/body of an attached clause", &[], vec![
            arg("number", "string", true, "Contract number"),
            arg("slug", "string", true, "Clause slug"),
        ], vec![
            opt("--heading", "string", "New heading"),
            opt("--body", "string", "New body markdown"),
            opt("--from-file", "string", "New body markdown from file"),
        ]),
        "contracts clauses remove": cmd("Remove a clause", &["contracts clauses rm"], vec![
            arg("number", "string", true, "Contract number"),
            arg("slug", "string", true, "Clause slug"),
        ], vec![]),
        "contracts clauses move": cmd("Re-order a clause", &[], vec![
            arg("number", "string", true, "Contract number"),
            arg("slug", "string", true, "Clause slug"),
            arg("position", "int", true, "New zero-indexed position"),
        ], vec![]),
        "contracts clauses reset": cmd("Reset the clause set to pack default", &[], number(), vec![]),
        "duplicate": cmd("Clone a contract as a fresh draft (end date cleared, term kept)", &["clone", "contracts duplicate"], number(), vec![
            opt("--client", "string", "Retarget to another client"),
            opt("--as", "string", "Retarget to another issuer"),
        ]),
        "delete": cmd("Delete a contract (draft only unless --force)", &["rm", "contracts delete"], number(), vec![
            opt("--force", "bool", "Allow deleting a non-draft contract"),
        ]),
        "pack list": cmd("List kind/pack combinations", &["pack ls", "packs list"], vec![], vec![]),
        "pack show": cmd("Show available clauses for a kind+pack", &["pack get"], vec![arg("kind", "string", true, "Contract kind")], vec![
            opt("--pack", "string", "Pack slug (default: standard)"),
        ]),
        "template list": cmd("List PDF templates with descriptions, moods, tags, fonts", &["template ls", "templates list"], vec![], vec![]),
        "template find": cmd("Rank templates against a free-text description of the look (local scoring; caller picks)", &["template suggest"], vec![
            arg("query", "string", true, "e.g. 'magazine masthead' or 'swiss minimal'"),
        ], vec![]),
        "template preview": cmd("Render a sample contract PDF with synthetic data", &[], vec![arg("name", "string", true, "Template name")], vec![
            opt("--kind", "string", "Contract kind to preview (default consulting)"),
            opt("--out", "string", "Output path"),
        ]),
        "kinds list": cmd("List contract kinds with descriptions and trigger tags", &["kinds ls", "kind list"], vec![], vec![]),
        "kinds find": cmd("Rank contract kinds against a free-text description of the need (local scoring; caller picks)", &["kinds suggest"], vec![
            arg("query", "string", true, "e.g. 'stop them going around me to my contact'"),
        ], vec![]),
        "config show": cmd("Display current configuration", &[], vec![], vec![]),
        "config path": cmd("Print the config file path", &[], vec![], vec![]),
        "config set": cmd("Set a config key", &[], vec![
            arg("key", "string", true, "default_issuer | default_template | open_pdf | self_update"),
            arg("value", "string", true, "New value"),
        ], vec![]),
        "agent-info": cmd("This manifest", &["info"], vec![], vec![]),
        "doctor": cmd("Check typst, DB, packs, templates, shared entities", &[], vec![], vec![]),
        "skill install": cmd("Install the embedded skill for Claude/Codex/Gemini", &[], vec![], vec![]),
        "skill status": cmd("Report where the skill is installed and whether it is current", &[], vec![], vec![]),
        "update": cmd("Distribution-aware update via brew or cargo", &[], vec![], vec![
            opt("--check", "bool", "Check only — never mutates"),
        ]),
    });

    let templates = crate::typst_assets::list_template_meta().unwrap_or_default();
    let packs = crate::clauses::list_packs();
    let kinds: Vec<serde_json::Value> = crate::kinds::KINDS
        .iter()
        .map(|k| json!({ "kind": k.kind, "description": k.description, "tags": k.tags }))
        .collect();

    let manifest = json!({
        "name": "contract",
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "commands": commands,
        "global_flags": {
            "--json": { "description": "Force JSON envelope output (auto-enabled when piped)", "type": "bool", "default": false },
            "--quiet": { "description": "Suppress informational human output", "type": "bool", "default": false },
        },
        "exit_codes": {
            "0": "Success",
            "1": "Transient error (IO, render) — retry",
            "2": "Config error — fix setup",
            "3": "Bad input / not found / ambiguous — fix arguments",
            "4": "Rate limited — wait and retry",
        },
        "envelope": {
            "version": "1",
            "success": "{ version, status, data }",
            "error": "{ version, status, error: { code, message, suggestion } }",
        },
        "config": {
            "path": config_path,
            "env_prefix": "PAPERFOOT_",
        },
        "auto_json_when_piped": true,
        "state_dir": state_dir,
        "database": database,
        "templates": templates,
        "packs": packs.into_iter().map(|(k, p)| json!({"kind": k, "pack": p})).collect::<Vec<_>>(),
        "kinds": kinds,
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
            "database": database,
            "shared_with": ["invoice-cli (binary: invoice)"],
            "shared_tables": ["issuers", "clients", "number_series"],
            "discovery_workflow": "Before creating a new issuer or client, ALWAYS run `contract issuer list --json` and `contract clients list --json` (or the invoice-cli equivalents). The accounting suite's whole point is one source of truth — duplicating entities here pollutes invoicing too. If the entity exists under a different slug, prefer using the existing slug over creating a new one.",
            "legal_fields_on_clients": "Three columns added in V7 are contract-specific: `legal_name`, `company_no`, `legal_jurisdiction`. If an existing client (from invoice-cli) is missing them, fill via: contract clients edit <slug> --legal-name X --company-no Y --jurisdiction Z."
        },
        "examples": [
            { "goal": "Quick mutual NDA",
              "command": "contract new --kind nda --as acme --client meridian --purpose 'evaluation of a joint product' --term-years 3" },
            { "goal": "Non-circumvention agreement protecting an introduction",
              "command": "contract new --kind ncnda --as boris --client partner --purpose 'introduction to prospective lenders for the transaction' --term-years 2" },
            { "goal": "Consulting agreement with fixed fee",
              "command": "contract new --kind consulting --as acme --client meridian --purpose 'design a dashboard' --fee fixed:8400:SGD --term-months 3 --deliverable 'Design' --deliverable 'Build'" },
            { "goal": "Interest-free loan with fixed repayment date",
              "command": "contract new --kind loan --as boris --client friend --term principal_text='£10,000 (ten thousand pounds sterling)' --term repayment_date=2026-12-01 --term interest_text='interest-free'" },
            { "goal": "Pick a template by describing the look",
              "command": "contract template find \"magazine masthead serif\"" },
            { "goal": "Render with no watermark",
              "command": "contract render NDA-acme-2026-0001 --final --open" },
        ],
        "guardrails": [
            "BEFORE creating any issuer or client, run `contract issuer list` and `contract clients list` — the DB is shared with invoice-cli; duplicates pollute both tools.",
            "Run doctor before first use.",
            "Use --json for agents; stdout is data, stderr is diagnostics.",
            "Drafts render with a DRAFT watermark by default. Use --final to suppress before sending out for signature.",
            "Once a contract is sent/signed, only the status / signature columns can change; executed contracts only move forward.",
            "These clauses are practical plain-English starting points, not legal advice. Have a lawyer review for material engagements.",
        ],
    });
    print_raw(&manifest);
    Ok(())
}
