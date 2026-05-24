// ═══════════════════════════════════════════════════════════════════════════
// vienna-legal — Warm boutique consulting contract.
// Cream paper, terracotta accent. Centred title.
//
// Real-contract conventions: kind-label IS the title; project name (if any)
// is the subtitle. "Dated 24 May 2026" line above PARTIES. Numbered (1)/(2)
// parties prose, not invoice-style two-column cards. Reference code only
// in the page footer for filing.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#1B1B1B"),
  paper:       rgb("#F5F0E6"),
  accent:      rgb("#B94735"),
  mute:        rgb("#6E685D"),
  hair:        rgb("#C7BFAE"),
  watermark:   rgb("#E6CFC1"),
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

// ─── HERO ──
#align(center)[
  #if data.logo != none [
    #image(data.logo, height: 7mm)
    #v(8pt)
  ]
  #fit-size(
    (24pt, 22pt, 20pt, 18pt),
    150mm,
    s => text(font: theme.display-font, size: s, weight: 700, tracking: 0pt, fill: theme.ink)[#data.kind-label],
  )
  #if data.subtitle != none [
    #v(6pt)
    #fit-size(
      (13pt, 12pt, 11pt),
      150mm,
      s => text(font: theme.display-font, size: s, weight: 400, fill: theme.mute)[#data.subtitle],
    )
  ]
  #v(8pt)
  #rect(width: 32mm, height: 1.4pt, fill: theme.accent, stroke: none)
]

#v(mm-sp.m)

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

// ─── KEY TERMS (docket row) ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#grid(
  columns: cells.map(_ => 1fr),
  column-gutter: 8mm,
  align: (left + horizon, left + horizon, left + horizon),
  ..cells.map(((lbl-t, val)) => [
    #text(size: 7.5pt, fill: theme.mute, tracking: 1pt)[#upper(lbl-t)]\
    #v(1pt)
    #text(size: 9.2pt)[#val]
  ])
)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.m)

// ─── AGREED TERMS heading ──
#section-label(theme, "Agreed terms")
#v(mm-sp.xs)

// ─── CLAUSES ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (8mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.display-font, size: 9.6pt, weight: 700, fill: theme.accent)[#clause.number.],
      [
        #text(font: theme.display-font, size: 10.4pt, weight: 700)[#clause.heading]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE (never splits across pages) ──
#v(mm-sp.l)
#signature-block(data.signature, theme)
