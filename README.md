# contract-cli

Draft business contracts and render them as carefully typeset PDFs. A local Rust CLI with versioned clause packs, selectable governing law, and a predictable JSON interface for agents.

Shares issuers and clients with [invoice-cli](https://github.com/paperfoot/invoice-cli) through [finance-core](https://github.com/paperfoot/finance-core).

## What is included

- Seven kinds: NDA, NCNDA, consulting, MSA, SOW, service and loan.
- Ten PDF templates, with A4 by default plus A3, A5, US Letter, Legal and Executive output. New **folio** for technology and startups, **counsel** for restrained legal typography, and **atelier** for design engagements.
- Standard clause packs plus `consulting/technology`, `consulting/design` and `msa/startup`.
- Explicit `global`, `uk`, `us` and `singapore` legal profiles.
- Draft editing, clause composition, lifecycle controls and administrative signature records.
- Local discovery with `kinds find` and `template find`; no model or network needed.
- JSON output when piped, semantic exit codes, and a capability manifest via `agent-info`.

## Build and install

Rust 1.88 or newer and Typst are required to build and render. The current dependencies are locked in `Cargo.lock`.

```sh
brew install typst
# Keep these repositories next to one another:
git clone https://github.com/paperfoot/finance-core.git
git clone https://github.com/paperfoot/contract-cli.git
cd contract-cli
cargo install --path . --locked
contract --version
```

The source build uses the sibling `finance-core` checkout. CI pins that dependency revision. The existing `paperfoot/tap` Homebrew distribution may lag the source branch; check the installed version before relying on new flags.

## Quick start

All entities below are fictional. Check `issuer list` and `clients list` first: their records are shared with invoice-cli.

```sh
contract issuer list
contract clients list

# Only create these if they do not already exist.
contract issuer add acme --name "Acme Example Ltd" --jurisdiction uk \
  --address '1 Example Street\nLondon' --email contracts@acme.example
contract clients add meridian --name "Meridian Example Ltd" \
  --address '2 Example Avenue\nLondon' --email legal@meridian.example

contract new --kind consulting --as acme --client meridian \
  --legal-profile uk --pack technology --template folio \
  --purpose "Build the customer dashboard" --term-months 3 \
  --fee fixed:8400:GBP --fee-schedule on-completion \
  --deliverable "Dashboard source code and build instructions" \
  --deliverable "Deployment and one handover session" \
  --term acceptance_days=10

# Use the number returned by new. Draft PDFs carry a DRAFT watermark.
contract render CTR-acme-2026-0001 --open
# A clean signing copy; this does not send or sign the document.
contract render CTR-acme-2026-0001 --final --paper us-letter --reference off
```

`--reference on|off` controls the internal reference printed in headers and footers (default: on). Page numbers remain visible. Paper changes reflow the document without scaling down its fonts.

The [typography specification](docs/TYPOGRAPHY.md) records the source guidance, actual font sizes, baseline spacing, page dimensions and review method.

## Choose the governing law

| Profile | Selection | Scope |
|---|---|---|
| UK | `--legal-profile uk` | Defaults to England and Wales; `--governing-law Scotland` or `"Northern Ireland"` also accepted. |
| US | `--legal-profile us --us-state Delaware` | Requires a full state name or District of Columbia. Adds a federal trade-secret immunity notice to current packs. |
| Singapore | `--legal-profile singapore` | Singapore governing law and courts unless an explicit venue is supplied. |
| Global | `--legal-profile global --governing-law Germany --venue "the courts of Berlin, Germany"` | A cross-border starting point requiring an explicit law and court venue. |

`--venue` is the complete court phrase inserted after “exclusive jurisdiction of”. It selects courts, not arbitration. Profiles select law, venue wording and limited notices; they do not provide a complete jurisdiction-specific legal adaptation. “Global” is not a governing law. See [legal scope and sources](docs/LEGAL.md).

Without a profile, `--governing-law` remains available. An omitted law falls back to the issuer's jurisdiction (UK resolves to England and Wales); an ambiguous US or EU default requires a specific law. Select a profile explicitly for new work. Editing law must remain consistent with the stored profile.

## Choose wording and design separately

| Clause pack | Additional provisions |
|---|---|
| `consulting/technology` | Objective acceptance, secure development, source and build handover, dependency licensing and AI use. |
| `consulting/design` | Revision allowance, source files, third-party assets, font licences and express portfolio permission. |
| `msa/startup` | Signed SOWs, continuity and exit handover; no implied equity or fundraising commitment. |

```sh
contract pack show consulting --pack design
contract template preview folio --kind consulting --pack technology --out ./technology.pdf
contract template preview counsel --kind nda --out ./nda.pdf
contract template preview atelier --kind consulting --pack design --out ./design.pdf
contract template find "quiet legal serif"
```

All ten templates use one shared layout: a 142 mm reading column on A4, hanging clause numbers, flush-left paragraphs with visible separation, aligned lists and intact signature blocks. The original `helvetica-nera`, `vienna-legal`, `editorial`, `gazette`, `marrakech`, `basel` and `chancery` names remain available with rebuilt layouts. The default remains `helvetica-nera`.

Template resolution is `--template`, then the contract's stored template, then a valid shared config template, then `helvetica-nera`. A shared invoice-only template is skipped. `config set default_template folio` changes the shared accounting configuration. Every stock template uses bundled OFL fonts. Body, headings, labels and signatures share the same family within each design; Gazette adds a separate display face for its title. The typography rules and review criteria are in [the design guide](docs/TYPOGRAPHY.md).

## Compose clauses

```sh
contract pack show nda
contract contracts clauses list NDA-acme-2026-0001
contract contracts clauses edit NDA-acme-2026-0001 purpose --from-file ./purpose.md
contract contracts clauses add NDA-acme-2026-0001 custom --heading "Project requirements" --body "Agreed wording."
contract contracts clauses reset NDA-acme-2026-0001
```

Bodies support plain paragraphs, dash bullets, numbered lists and wrapped list items. Clause text is treated as text, not executable Typst. Pack variables use `{{name}}`; scalar terms can be supplied with repeatable `--term key=value`. Recognised numeric and choice terms are validated. `legal_profile` is reserved for `--legal-profile`.

A SOW needs `--term msa_reference="MSA number and date"`. Loan terms include `principal_text`, `interest_text` and `repayment_date`. Supply complete terms and review the rendered agreement: unresolved variables block a clean render, but the CLI does not establish commercial completeness or enforceability.

New contracts snapshot selected clause templates and store the pack version. Archived standard packs preserve the immediately preceding shipped versions. Older contracts resolve against their stored version; an unavailable version fails explicitly. Reset uses that same version. Duplicate retains the source wording, law, venue and terms, clears signatures and the absolute end date, and refreshes an automatically generated title for new parties. Review copied dates embedded in free-form terms.

## Lifecycle and signatures

`draft → sent → signed → active → expired / terminated`

Sent contracts lock metadata and clauses. An unsigned sent contract can return to draft. Recording even one signature locks edits and prevents recall. Executed contracts cannot return to an editable state. Terminated and expired contracts cannot receive signatures.

```sh
contract mark NDA-acme-2026-0001 sent
# Record details only after signing has happened through the agreed process.
contract sign NDA-acme-2026-0001 --side us --name "Alex Morgan" --title Director
contract sign NDA-acme-2026-0001 --side them --name "Sam Taylor" --title Director
```

`sign` records names and dates; it does not authenticate people, obtain consent, send a document, cryptographically sign a PDF, or provide an e-signature audit trail. `mark signed` is an administrative assertion. Retain the actual executed PDF and its evidence separately. Rerendering is not archival reproduction: party records, formatting and software can change.

## State and privacy

Config and SQLite state live in the accounting suite's platform-specific application directory; use `config path` and `agent-info` to inspect the locations. Templates and fonts are cached in its assets directory.

Drafting and rendering are local. There is no telemetry or document upload. Explicit update commands use the network; `--open` launches the local PDF viewer. The database, exported PDFs, command JSON and shell history can contain party information. Internal contract notes are excluded from the rendering sidecar and PDF. Database files, environment files and generated PDFs are ignored by Git. This repository's examples use fictional entities and `.example` email addresses.

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
# Requires Typst and Poppler (brew install typst poppler).
python3 scripts/smoke-pdfs.py --binary target/debug/contract
```

The PDF smoke check renders every template/kind combination and the specialised packs, verifies actual clause text, text bounds and unresolved variables, and checks US Letter dimensions when `pdfinfo` is available. Long-party cases verify wrapping and full legal names. A long custom-document case also checks wrapped lists, private-note exclusion, missing-term rejection and preservation of an existing PDF after compiler failure. Tests isolate HOME and XDG state. The migration tests preserve populated legacy records, custom indexes/triggers, foreign keys and sequence values.

See [changes](CHANGELOG.md) and [legal scope](docs/LEGAL.md). These are drafting starting points for two-party business agreements, not a substitute for legal advice or prescribed regulated documents.

MIT © 199 Biotechnologies
