# contract-cli

> Beautiful contracts from the CLI — NDA, consulting, MSA, SOW, service.
> Plain English, 1–3 pages, agent-friendly.

A stateful, single-binary CLI for drafting and rendering business contracts.
Built for humans who want a clean terminal workflow *and* for AI agents that
need a deterministic JSON interface to draft and dispatch contracts on
their owner's behalf.

Sibling of [`invoice-cli`](https://github.com/paperfoot/invoice-cli) — same
SQLite store, same issuers, same clients. Drop a contract draft for a
client you already invoice, in one binary.

## Features

- **Five contract kinds, plain English.** Consulting, NDA (mutual or
  unilateral), MSA, SOW, service agreement. Embedded clause packs use
  real-contract conventions: kind as title, project as subtitle, "Dated"
  line, numbered `(1)/(2)` parties prose, "AGREED TERMS" section.
- **Composable clause packs.** Each kind ships with a `standard` pack.
  Include or exclude clauses at creation time (`--include non_solicit
  --exclude warranties`), or override the body of any clause from a
  Markdown file (`contract clauses edit <num> termination --from-file
  ./my-version.md`). Reset to pack default with `contract clauses reset
  <num>`. Add custom clauses entirely (`contract clauses add <num>
  custom_audit --body "…"`).
- **Three Typst templates.** `helvetica-nera` (Swiss monochrome, default),
  `vienna-legal` (Bauhaus / terracotta), `editorial` (centred serif —
  reads like a deed). Each template owns the masthead aesthetic;
  shared/contract.typ owns the structure (parties prose, key terms,
  numbered clauses, signature block).
- **Shared with invoice-cli.** Issuers + clients live in one accounting
  SQLite at `~/Library/Application Support/com.paperfoot.accounting/`.
  Anything you `invoice clients add ...` is immediately usable by
  `contract new ... --client …`, and vice versa. `contract doctor`
  surfaces the shared list and warns when a client lacks the legal
  fields needed for a contract party block.
- **Signature lifecycle.** `draft → sent → signed → active / expired /
  terminated`. Drafts render with a faint DRAFT watermark; `--final`
  produces a clean signing copy. `contract sign <num> --side us|them
  --name "..." --title "..."` records each signature; status
  auto-promotes to `signed` when both sides have signed. Sent/signed
  contracts are immutable — clauses and metadata lock.
- **Signature block never splits.** If the execution block doesn't fit
  on the current page, Typst pushes the whole block to the next page —
  never half on one page, half on another.
- **Agent-friendly.** Every command emits a `{version, status, data |
  error}` envelope when piped or `--json`. `contract agent-info` returns
  a full capability + exit-code manifest with a `shared_state` block
  telling the agent which entities are shared with invoice-cli and to
  list existing before creating duplicates. `contract skill install`
  drops a ready-to-use skill into `~/.claude/skills/contract-cli/`,
  `~/.codex/skills/contract-cli/`, `~/.gemini/skills/contract-cli/`.

## Install

### Homebrew (macOS / Linux)

```
brew tap paperfoot/tap
brew install contract
```

### Cargo

```
cargo install contract-cli
```

### From source

```
git clone https://github.com/paperfoot/contract-cli
cd contract-cli
cargo install --path .
```

All install paths produce a single `contract` binary. Typst is the only
runtime dependency (`brew install typst` on macOS).

## Quick start

```sh
# Reuse the issuers and clients you already have in invoice-cli, or add new
contract issuer list                                # shared with invoice-cli
contract clients list

# Add legal fields to an existing client (V7 added these, invoice-cli
# doesn't need them but contracts do)
contract clients edit reshape-clinic \
    --legal-name "Reshape Clinic Ltd" \
    --company-no "12345678" \
    --jurisdiction "England and Wales"

# Quick mutual NDA — 3-year term
contract new --kind nda --as boris --client reshape-clinic \
    --purpose "exploring a joint product line" --term-years 3

# Consulting agreement, fixed fee, plain-English clause pack
contract new --kind consulting --as boris --client alberto-pertusa \
    --title "Superlearning Ltd — Company Formation Engagement" \
    --purpose "guide and execute the registration of Superlearning Ltd at Companies House and stand up the company's initial governance, share, and tax setup" \
    --fee fixed:1500:GBP --fee-schedule on-completion \
    --term-months 2 \
    --deliverable "Form IN01 filing at Companies House" \
    --deliverable "Bespoke Articles of Association" \
    --deliverable "Initial share structure + statutory registers" \
    --deliverable "HMRC Corporation Tax registration" \
    --ip-assignment client \
    --governing-law "England and Wales"

# Render — DRAFT watermark by default; --final removes it
contract render CTR-boris-2026-0001 --template editorial --final --open

# Record signatures — status auto-bumps to "signed" when both sides done
contract sign CTR-boris-2026-0001 --side us   --name "B. Djordjevic" --title "Director"
contract sign CTR-boris-2026-0001 --side them --name "A. Pertusa"
```

## Core commands

| Command | Purpose |
|---|---|
| `issuer add\|edit\|list\|show\|delete` | Manage issuers (your side; shared with invoice-cli) |
| `clients add\|edit\|list\|show\|delete` | Manage counterparties (shared with invoice-cli; V7 adds `legal_name`, `company_no`, `legal_jurisdiction`) |
| `new --kind <k> --as <i> --client <c> [options...]` | Create a new contract |
| `list [--kind --status --as]` | List contracts |
| `show <number>` | Show metadata + clause list |
| `contracts edit <number>` | Edit draft metadata (sent/signed are immutable) |
| `render <number> [--template] [--out] [--open] [--final \| --draft]` | Generate PDF. DRAFT watermark by default; `--final` for clean copy |
| `mark <number> draft\|sent\|signed\|active\|expired\|terminated` | Update status — auto-stamps timestamps |
| `sign <number> --side us\|them --name "..." [--title] [--date]` | Record one party's signature |
| `contracts clauses list\|add\|edit\|remove\|move\|reset <number>` | Compose the clause set |
| `contracts duplicate <number>` | Clone a contract as a fresh draft |
| `contracts delete <number> [--force]` | Delete (`--force` for non-draft) |
| `pack list \| show <kind>` | Browse the clause packs |
| `template list \| preview <name> [--kind nda\|consulting]` | Inspect templates |
| `doctor` | Verify typst + DB + packs + shared issuers/clients |
| `agent-info` | Full JSON capability manifest |
| `skill install` | Install embedded Claude / Codex / Gemini skill |
| `update [--check]` | Self-update via brew or cargo |

Run `contract --help` for the full reference.

## Template resolution

At render time the chain is:

```
--template flag  >  contract.default_template  >  "helvetica-nera"
```

## Composing clauses

Each contract is built from a clause pack. By default it includes the
pack's `default_clauses` in order. Customise per contract:

```
contract pack show consulting
contract contracts clauses list CTR-boris-2026-0001
contract contracts clauses add CTR-boris-2026-0001 non_solicit --from-file ./extra.md --position 8
contract contracts clauses edit CTR-boris-2026-0001 termination --from-file ./our-termination.md
contract contracts clauses remove CTR-boris-2026-0001 warranties
contract contracts clauses move CTR-boris-2026-0001 governing_law 12
contract contracts clauses reset CTR-boris-2026-0001    # back to pack default
```

A clause's body uses simple Markdown — paragraphs, dash bullets, and
`1.` numbered lists. Pack clauses use `{{vars}}` like `{{our_legal_name}}`,
`{{their_legal_name}}`, `{{effective_date}}`, `{{term_text}}`,
`{{governing_law}}`, `{{jurisdiction_phrase}}`, `{{fee_text}}`,
`{{deliverables_block}}`, `{{ip_assignment_text}}`,
`{{confidentiality_years}}`, `{{termination_notice_days}}`,
`{{purpose}}`.

## State & privacy

- **Config:** shared Paperfoot accounting config (`~/Library/Application
  Support/com.paperfoot.accounting/config.toml` on macOS).
- **Database:** shared SQLite at `accounting.db` in the same dir.
- **Templates:** extracted on first use to the shared assets dir;
  refreshed on upgrade.

Nothing ever leaves your machine. No telemetry. No phone-home.

## Architecture

- **Rust** binary via `cargo` / single-binary distribution.
- **SQLite** via `rusqlite` + `refinery` migrations
  (`finance-core/migrations/V7__contracts.sql`).
- **Typst** for PDF rendering — templates embedded via `rust-embed`, JSON
  sidecar pattern (Rust builds `ContractRenderData`, writes it next to
  the templates in a temp dir, `typst compile` loads the JSON).
- **Clause packs** as TOML, embedded via `rust-embed`. Five kinds × one
  `standard` pack at launch.
- **Built on** [`finance-core`](https://github.com/paperfoot/finance-core)
  and follows the [`agent-cli-framework`](https://github.com/199-biotechnologies/agent-cli-framework)
  conventions for agent ergonomics.

## Scope

This is a **contract drafting tool**, not legal advice. In scope:

- Clean, plain-English contract generation across common business kinds
- Composable clauses (include / exclude / override / custom)
- Lifecycle tracking (draft → sent → signed → active → expired /
  terminated) with signature records
- Multiple Typst templates with three distinct voices
- Shared accounting state with invoice-cli (issuers, clients)

Explicitly out of scope:

- Negotiation workflow / redlines / version diffing
- E-signature platforms (DocuSign, Dropbox Sign) — out for v1
- M&A documents, court filings, regulatory submissions
- Legal advice — these are practical starting points; have a lawyer
  review for material engagements

## License

MIT © 199 Biotechnologies
