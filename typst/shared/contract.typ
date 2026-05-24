// ═══════════════════════════════════════════════════════════════════════════
// Shared helpers — loaded by every template.
//
// Loads the Rust-generated `contract.json` and exposes the contract data as
// `data`. Provides minimal markdown rendering (paragraphs + dash/numbered
// lists), a party block, signature block, and an optional draft watermark
// stamp. Templates own the visual identity (colors, fonts, layout); this
// file owns the content shape.
// ═══════════════════════════════════════════════════════════════════════════

#let data = json("contract.json")

// ─── Minimal markdown → Typst rendering ───────────────────────────────────
//
// Handles three things:
//   1. Paragraph breaks (blank line)
//   2. Bullet lists: lines starting with "- "
//   3. Numbered lists: lines starting with "1. ", "2. ", etc.
//
// Anything else is rendered as flowing paragraph text. Inline punctuation is
// passed through unchanged — Typst already handles curly quotes / em-dashes
// because the input strings use them literally.

// Returns true if `s` starts with `<digits>. ` (e.g. "1. ", "12. ").
#let starts-with-number(s) = {
  let m = s.match(regex("^\d+\.\s"))
  m != none
}

#let strip-num-prefix(s) = {
  let m = s.match(regex("^\d+\.\s+"))
  if m == none { s } else { s.slice(m.end) }
}

#let render-markdown(md) = {
  // Split into paragraph blocks on blank lines.
  let paras = md.split("\n\n").map(p => p.trim())
  for p in paras {
    if p == "" { continue }
    let lines = p.split("\n")
    let first = lines.find(l => l.trim() != "")
    if first == none { continue }
    let first-t = first.trim()
    if first-t.starts-with("- ") {
      list(
        ..lines
          .filter(l => l.trim().starts-with("- "))
          .map(l => l.trim().slice(2))
      )
    } else if starts-with-number(first-t) {
      enum(
        ..lines
          .filter(l => starts-with-number(l.trim()))
          .map(l => strip-num-prefix(l.trim()))
      )
    } else {
      let joined = lines.map(l => l.trim()).filter(l => l != "").join(" ")
      par(joined)
    }
  }
}

// ─── Party block ──────────────────────────────────────────────────────────

#let party-block(party, theme) = {
  set text(size: 9pt)
  // Role label as a small tracked uppercase header
  text(size: 7.5pt, fill: theme.mute, tracking: 1pt, weight: 500)[#upper(party.role-label)]
  v(2pt)
  text(size: 10.5pt, weight: 600)[#party.display-name]
  if party.legal-name != none and party.legal-name != party.display-name [
    \
    #text(size: 9pt, fill: theme.mute)[#party.legal-name]
  ]
  if party.company-no != none [
    \
    #text(size: 8.5pt, fill: theme.mute)[Reg. no. #party.company-no]
  ]
  if party.jurisdiction != none [
    \
    #text(size: 8.5pt, fill: theme.mute)[A company of #party.jurisdiction]
  ]
  v(3pt)
  for line in party.address {
    text(size: 9pt)[#line]
    linebreak()
  }
  if party.attn != none [
    #v(2pt)
    #text(size: 8.5pt, fill: theme.mute)[Attn: #party.attn]
  ]
  if party.email != none [
    #v(2pt)
    #text(size: 8.5pt, fill: theme.mute)[#party.email]
  ]
}

// ─── Signature block (two columns, plenty of space) ───────────────────────

#let signature-pair(label, party-name, signer-name, signer-title, signer-date, theme) = {
  set text(size: 9pt)
  text(size: 7.5pt, fill: theme.mute, tracking: 1pt, weight: 500)[#upper("Signed for and on behalf of")]
  v(3pt)
  text(size: 10pt, weight: 600)[#party-name]
  v(18mm)
  line(length: 100%, stroke: 0.4pt + theme.ink)
  v(2pt)
  grid(
    columns: (auto, 1fr, auto),
    column-gutter: 2mm,
    text(size: 7.5pt, fill: theme.mute, tracking: 0.8pt)[NAME],
    text(size: 9pt)[#if signer-name != none { signer-name } else { "" }],
    [],
  )
  v(2pt)
  if signer-title != none [
    #grid(
      columns: (auto, 1fr, auto),
      column-gutter: 2mm,
      text(size: 7.5pt, fill: theme.mute, tracking: 0.8pt)[TITLE],
      text(size: 9pt)[#signer-title],
      [],
    )
    #v(2pt)
  ]
  grid(
    columns: (auto, 1fr, auto),
    column-gutter: 2mm,
    text(size: 7.5pt, fill: theme.mute, tracking: 0.8pt)[DATE],
    text(size: 9pt)[#if signer-date != none { signer-date } else { "" }],
    [],
  )
}

#let signature-block(sig, theme) = {
  grid(
    columns: (1fr, 1fr),
    column-gutter: 14mm,
    signature-pair(
      sig.our-label,
      sig.our-name,
      sig.at("our-signer-name", default: none),
      sig.at("our-signer-title", default: none),
      sig.at("our-signer-date", default: none),
      theme,
    ),
    signature-pair(
      sig.their-label,
      sig.their-name,
      sig.at("their-signer-name", default: none),
      sig.at("their-signer-title", default: none),
      sig.at("their-signer-date", default: none),
      theme,
    ),
  )
}

// ─── Draft watermark (diagonal large-text overlay) ────────────────────────

#let draft-watermark(text-color) = {
  place(
    center + horizon,
    dx: 0pt,
    dy: 0pt,
    rotate(
      -28deg,
      text(
        size: 110pt,
        weight: 700,
        fill: text-color,
        tracking: 6pt,
      )[DRAFT],
    ),
  )
}

// ─── Clauses block ────────────────────────────────────────────────────────

#let clauses-block(clauses, theme, number-style: "decimal", heading-style: "side") = {
  for clause in clauses {
    block(breakable: true, spacing: 6mm, [
      #grid(
        columns: (10mm, 1fr),
        column-gutter: 4mm,
        align: (right + top, left + top),
        text(size: 10pt, weight: 600, fill: theme.accent)[#clause.number.],
        [
          #text(size: 10.5pt, weight: 600, fill: theme.ink)[#clause.heading]
          #v(2pt)
          #render-markdown(clause.body)
        ],
      )
    ])
  }
}
