// ═══════════════════════════════════════════════════════════════════════════
// helvetica-nera — Sober corporate/legal instrument.
// Left-aligned engagement name leads, kind acts as subtitle in mute.
// Real-contract conventions: no eyebrow, no reference code in body,
// "DATED" line + numbered (1)/(2) parties prose, full-black clauses.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#111111"),
  paper:       rgb("#FFFFFF"),
  accent:      rgb("#111111"),
  mute:        rgb("#666666"),
  hair:        rgb("#D7D7D7"),
  watermark:   rgb("#D8D8D8"),
  display-font: ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  body-font:    ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 24mm, bottom: 22mm, left: 24mm, right: 24mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 9.6pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  number-width: "tabular",
  hyphenate: false,
)
#set par(leading: 5.6pt, spacing: 5.6pt, justify: true)

// ─── HERO (left-aligned, the engagement leads) ──
#fit-size(
  (22pt, 20pt, 19pt, 18pt),
  155mm,
  s => text(font: theme.display-font, size: s, weight: 700, tracking: -0.2pt)[#data.kind-label],
)
#if data.subtitle != none [
  #v(4pt)
  #text(font: theme.display-font, size: 12pt, weight: 400, fill: theme.mute)[#data.subtitle]
]

#v(mm-sp.s)
#line(length: 100%, stroke: 0.6pt + theme.ink)
#v(mm-sp.s)

// ─── DATED ──
#section-label(theme, "Dated") #h(6pt) #text(size: 9.6pt)[#data.effective-date-display]

#v(mm-sp.s)

// ─── PARTIES ──
#section-label(theme, "Parties")
#v(2pt)
#parties-prose-block(data.parties-prose, theme)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.s)

// ─── KEY TERMS ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#grid(
  columns: cells.map(_ => 1fr),
  column-gutter: 8mm,
  align: (left + horizon, left + horizon, left + horizon),
  ..cells.map(((lbl-t, val)) => [
    #text(size: 7.5pt, fill: theme.mute, tracking: 1.2pt)[#upper(lbl-t)]\
    #v(1pt)
    #text(size: 9.5pt)[#val]
  ])
)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.m)

// ─── AGREED TERMS ──
#section-label(theme, "Agreed terms")
#v(mm-sp.xs)

// ─── CLAUSES (mute number, no accent) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (8mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.display-font, size: 10pt, weight: 600, fill: theme.mute)[#clause.number.],
      [
        #text(font: theme.display-font, size: 10.5pt, weight: 700)[#clause.heading]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE ──
#v(mm-sp.l)
#signature-block(data.signature, theme)
