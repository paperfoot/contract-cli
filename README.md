# contract-cli

> Beautiful contracts from the CLI — NDA, NCNDA, consulting, MSA, SOW, service, loan.
> Plain English, 1–3 pages, agent-friendly.

A stateful, single-binary CLI for drafting and rendering business contracts.
Built for humans who want a clean terminal workflow *and* for AI agents that
need a deterministic JSON interface to draft and dispatch contracts on
their owner's behalf.

Sibling of [`invoice-cli`](https://github.com/paperfoot/invoice-cli) — same
SQLite store, same issuers, same clients. Drop a contract draft for a
client you already invoice, in one binary.

## Features

- **Seven contract kinds, plain English.** NDA (mutual or unilateral),
  NCNDA (non-circumvention — protects introductions from being cut out),
  consulting, MSA, SOW, service, loan. Embedded clause packs use
  real-contract conventions: kind as title, project as subtitle, "Dated"
  line, numbered `(1)/(2)` parties prose, "AGREED TERMS" section, discrete
  boilerplate (notices, counterparts & e-signatures, third-party rights,
  no-partnership, entire agreement).
- **Describe what you need — get the right kind.** `contract kinds find
  "stop them going around me to steal my contact"` → `ncnda`. Deterministic
  local scoring over per-kind trigger tags; ranked candidates, you pick.
  No LLM, no network, no guessing.
- **Seven Typst templates, described and findable.** `contract template
  list` shows each template's description, mood, tags, and fonts;
  `contract template find "magazine masthead"` ranks them. The lineup:
  - `helvetica-nera` — sober Swiss corporate instrument (default)
  - `vienna-legal` — warm boutique, cream + terracotta
  - `editorial` — formal serif, reads like a deed
  - `gazette` — magazine masthead: Fraunces black over a broadsheet
    dateline, literary Newsreader body
  - `marrakech` — the Iowan / cream / terracotta data-room voice
  - `basel` — Swiss brutalist grid with a marginalia rail
  - `chancery` — engraved deed: letterspaced Garamond capitals, true
    small caps, witness-ready execution
- **Premium fonts ship in the binary.** OFL faces (Fraunces, Newsreader,
  Literata, Libre Franklin, Archivo, EB Garamond, Cormorant Garamond)
  are embedded and passed to Typst via `--font-path`, so renders are
  identical on any machine. System faces (Iowan Old Style, Helvetica
  Neue) sit first in the stacks with embedded fallbacks.
- **Composable clause packs.** Include or exclude clauses at creation
  time (`--include non_circumvention --exclude warranties`), override any
  clause body from Markdown, add fully custom clauses. Arbitrary terms
  flow into pack `{{vars}}` via `--term key=value` — e.g.
  `--term principal_text='£10,000 (ten thousand pounds sterling)'`.
- **Signature lifecycle with teeth.** `draft → sent → signed → active →
  expired / terminated`. Executed contracts only move forward; `sign`
  refuses to overwrite a recorded signature without `--force`; drafts
  render with a DRAFT watermark (`--final` for the signing copy);
  sent/signed contracts lock clauses and metadata.
- **Agent-native, conformance-tested.** Full
  [agent-cli-framework](https://github.com/paperfoot/agent-cli-framework)
  compliance, verified in CI by the framework's conformance script:
  canonical `agent-info` manifest with per-command arg/option schemas,
  JSON envelope on every code path (piped `--help` included), semantic
  exit codes 0–4, tested error suggestions, distribution-aware `update`,
  `skill install|status` for Claude / Codex / Gemini.

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

All install paths produce a single `contract` binary. Typst is the only
runtime dependency (`brew install typst`).

## Quick start

```sh
contract issuer list                                # shared with invoice-cli
contract clients list

# Don't remember the kind or template names? Describe them:
contract kinds find "they can't bypass me and deal direct"
contract template find "warm cream editorial"

# Quick mutual NDA — 3-year term
contract new --kind nda --as boris --client reshape-clinic \
    --purpose "exploring a joint product line" --term-years 3

# Non-circumvention agreement protecting an introduction
contract new --kind ncnda --as boris --client partner \
    --purpose "introduction to prospective lenders in connection with the transaction" \
    --term-years 2 --governing-law "England and Wales"

# Interest-free loan, fixed repayment date
contract new --kind loan --as boris --client friend \
    --term principal_text='£10,000 (ten thousand pounds sterling)' \
    --term interest_text='interest-free' \
    --term repayment_date=2026-12-01

# Render — DRAFT watermark by default; --final removes it
contract render NDA-boris-2026-0001 --template marrakech --final --open

# Record signatures — status auto-bumps to "signed" when both sides sign
contract sign NDA-boris-2026-0001 --side us   --name "B. Djordjevic" --title "Director"
contract sign NDA-boris-2026-0001 --side them --name "A. Pertusa"
```

## Core commands

| Command | Purpose |
|---|---|
| `issuer add\|edit\|list\|show\|delete` | Manage issuers (your side; shared with invoice-cli) |
| `clients add\|edit\|list\|show\|delete` | Manage counterparties (shared with invoice-cli) |
| `new --kind <k> --as <i> --client <c> [options…]` | Create a contract (`--term key=value` for kind-specific terms) |
| `list \| show \| edit \| duplicate \| delete` | Work the contract book (all top-level) |
| `render <number> [--template] [--out] [--open] [--final \| --draft]` | Generate the PDF |
| `mark <number> <status>` | Lifecycle moves (forward-only once executed) |
| `sign <number> --side us\|them --name "…"` | Record a signature (`--force` to overwrite) |
| `contracts clauses list\|add\|edit\|remove\|move\|reset` | Compose the clause set |
| `pack list \| show <kind>` | Browse clause packs |
| `template list \| find "<look>" \| preview <name>` | Discover and preview templates |
| `kinds list \| find "<need>"` | Discover contract kinds |
| `doctor` | Verify typst + DB + packs + shared issuers/clients |
| `agent-info` | Canonical JSON capability manifest |
| `skill install \| status` | Manage the embedded agent skill |
| `update [--check]` | Distribution-aware update (brew / cargo) |

Run `contract --help` for Tips and Examples.

## Template resolution

```
--template flag  >  contract.default_template  >  "helvetica-nera"
```

Templates are validated when set, not just at render. Every template
carries a `//!` metadata header (description, mood, tags, fonts, paper) —
drop your own `.typ` in the extracted templates dir with the same header
and it participates in `template list` / `template find` immediately.

## Composing clauses

```
contract pack show ncnda
contract contracts clauses add NCNDA-boris-2026-0001 non_solicit --position 8
contract contracts clauses edit NCNDA-boris-2026-0001 termination --from-file ./ours.md
contract contracts clauses reset NCNDA-boris-2026-0001
```

Clause bodies use simple Markdown — paragraphs, dash bullets, `1.`
numbered lists. Pack clauses substitute `{{vars}}`: the built-ins
(`{{our_legal_name}}`, `{{effective_date}}`, `{{term_text}}`,
`{{governing_law}}`, `{{jurisdiction_phrase}}`, `{{fee_text}}`,
`{{deliverables_block}}`, `{{purpose}}`, …) plus any scalar you set via
`--term key=value`.

## State & privacy

- **Config:** `~/Library/Application Support/com.paperfoot.accounting/config.toml`
  (macOS), env overrides via `PAPERFOOT_*`.
- **Database:** shared SQLite `accounting.db` in the same dir.
- **Templates & fonts:** extracted to the shared assets dir on first use;
  refreshed on upgrade.

Nothing ever leaves your machine. No telemetry. No phone-home.

## Architecture

- **Rust** single binary; **SQLite** via `rusqlite` + `refinery`
  migrations (shared [`finance-core`](https://github.com/paperfoot/finance-core)).
- **Typst** renders the PDF — templates + OFL fonts embedded via
  `rust-embed`, JSON sidecar pattern, `typst compile --font-path`.
- **Clause packs** as TOML, embedded. Seven kinds × `standard` pack.
- Follows [`agent-cli-framework`](https://github.com/paperfoot/agent-cli-framework);
  the conformance script runs in CI.

## Scope

A **contract drafting tool**, not legal advice. Out of scope for now:
negotiation/redline workflow, e-signature platforms, contracts with more
than two parties (multi-party NCNDA/JV support is planned — the data
model groundwork exists), M&A/court/regulatory documents. Have a lawyer
review anything material.

## License

MIT © 199 Biotechnologies
