//! description: Data-room deal voice — Iowan Old Style on cream, terracotta section labels, ruled key-terms table. Patrician and warm, built for sophisticated counterparties.
//! mood: warm, patrician, editorial
//! tags: iowan, cream, terracotta, data-room, warm, deal-memo, patrician, editorial
//! fonts: Iowan Old Style (system), Literata (embedded OFL fallback)
//! paper: cream
// ═══════════════════════════════════════════════════════════════════════════
// marrakech — the Iowan/cream/terracotta data-room voice.
// Tokens from the Marrakech brief: cream #F6F1E7 paper, near-black ink,
// one terracotta accent, hairline rules, tracked uppercase labels, weights
// capped at 500-600. Key terms set as a proper ruled table. No decoration
// the brief didn't have.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#1D1A16"),
  paper:       rgb("#F6F1E7"),
  accent:      rgb("#8A3A1F"),
  mute:        rgb("#6B665E"),
  hair:        rgb("#D9D2C5"),
  watermark:   rgb("#E4D8C4"),
  display-font: ("Iowan Old Style", "Literata", "Georgia"),
  body-font:    ("Iowan Old Style", "Literata", "Georgia"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 26mm, bottom: 24mm, left: 26mm, right: 26mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 10.2pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  hyphenate: true,
)
#set par(leading: 6.4pt, spacing: 6.4pt, justify: true)

// Terracotta section label — the h2 of the Marrakech system.
#let mk-label(txt) = text(
  font: theme.display-font, size: 9pt, weight: 500,
  tracking: 1.6pt, fill: theme.accent,
)[#upper(txt)]

// ─── TITLE ──
#fit-size(
  (23pt, 21pt, 19pt, 17pt),
  158mm,
  s => text(font: theme.display-font, size: s, weight: 500, tracking: 0pt)[#data.kind-label],
)
#if data.subtitle != none [
  #v(4pt)
  #text(font: theme.display-font, size: 12pt, style: "italic", fill: theme.mute)[#data.subtitle]
]

#v(mm-sp.s)
#hairline(theme, weight: 0.4pt)
#v(mm-sp.s)

// ─── DATED ──
#mk-label("Dated") #h(6pt) #text(size: 10.2pt)[#data.effective-date-display]

#v(mm-sp.s)

// ─── PARTIES ──
#mk-label("Parties")
#v(2pt)
#parties-prose-block(data.parties-prose, theme)

#v(mm-sp.m)

// ─── KEY TERMS (proper ruled table) ──
#mk-label("Key terms")
#v(3pt)
#let terms = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  terms = terms + (("Fee", data.fee-short),)
}
#let n-terms = terms.len()
#block(breakable: false, table(
  columns: (42mm, 1fr),
  stroke: (x, y) => (
    top: if y == 0 { 0.5pt + theme.hair } else { 0.3pt + theme.hair },
    bottom: if y == n-terms - 1 { 0.5pt + theme.hair } else { none },
  ),
  inset: (x: 0pt, y: 5.5pt),
  align: (left + horizon, left + horizon),
  ..terms
    .map(((lbl-t, val)) => (
      text(size: 8pt, fill: theme.mute, tracking: 1.2pt)[#upper(lbl-t)],
      text(size: 10.2pt)[#val],
    ))
    .flatten()
))

#v(mm-sp.m)

// ─── AGREED TERMS ──
#mk-label("Agreed terms")
#v(mm-sp.xs)

// ─── CLAUSES (terracotta number, serif heading at weight 500) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (8mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.display-font, size: 11pt, weight: 500, fill: theme.accent)[#clause.number.],
      [
        #text(font: theme.display-font, size: 11pt, weight: 600)[#clause.heading]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE (closing rule travels with the block) ──
#v(mm-sp.l)
#block(breakable: false, [
  #hairline(theme, weight: 0.4pt)
  #v(mm-sp.s)
  #signature-block(data.signature, theme)
])
