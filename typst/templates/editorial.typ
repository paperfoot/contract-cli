// ═══════════════════════════════════════════════════════════════════════════
// editorial — Formal letter / private legal instrument.
// Centred serif. Roman semibold title (not italic). Italic only for the
// "between A and B" subtitle. Generous leading. Reads like a deed.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, party-block, clauses-block, signature-block, page-shell, sp, mm-sp, render-markdown

#let theme = (
  ink:         rgb("#1D1A16"),
  paper:       rgb("#F8F4EA"),
  accent:      rgb("#2A2A2A"),
  mute:        rgb("#7A7267"),
  hair:        rgb("#D6CBB9"),
  watermark:   rgb("#E5DCC9"),
  display-font: ("Georgia", "Newsreader", "EB Garamond", "Times New Roman"),
  body-font:    ("Georgia", "Newsreader", "EB Garamond", "Times New Roman"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 28mm, bottom: 26mm, left: 30mm, right: 30mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 10.2pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  hyphenate: false,
)
#set par(leading: 6.6pt, spacing: 6.6pt, justify: true, first-line-indent: 0pt)

// ─── HERO (centred) ──
#align(center)[
  #fit-size(
    (24pt, 22pt, 20pt, 18pt),
    132mm,
    s => text(font: theme.display-font, size: s, weight: 600)[#data.title],
  )
  #v(8pt)
  #text(size: 11pt, fill: theme.mute, style: "italic")[
    between #data.our-party.legal-name and #data.their-party.legal-name
  ]
  #v(8pt)
  #line(length: 28mm, stroke: 0.4pt + theme.hair)
  #v(6pt)
  #text(size: 9pt, fill: theme.mute, style: "italic")[
    #data.kind-label · Effective #data.effective-date-display · № #data.number
  ]
]

#v(mm-sp.m)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.s)
#align(center, line(length: 28mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.s)

// ─── KEY TERMS — docket row, hair rules ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#line(length: 100%, stroke: 0.3pt + theme.hair)
#pad(top: 5pt, bottom: 5pt)[
  #grid(
    columns: cells.map(_ => 1fr),
    column-gutter: 8mm,
    align: (left + horizon, left + horizon, left + horizon),
    ..cells.map(((lbl-t, val)) => [
      #text(size: 7.5pt, fill: theme.mute, tracking: 1pt, style: "italic")[#upper(lbl-t)]\
      #v(1pt)
      #text(size: 9.6pt)[#val]
    ])
  )
]
#line(length: 100%, stroke: 0.3pt + theme.hair)

#v(mm-sp.m)

// ─── CLAUSES (serif, oldstyle figures where available) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (8mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.display-font, size: 11pt, weight: 500)[#clause.number.],
      [
        #text(font: theme.display-font, size: 11pt, weight: 600)[#clause.heading]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE ──
#v(mm-sp.l)
#align(center, text(size: 9pt, tracking: 2pt, fill: theme.mute, style: "italic")[#upper("In witness whereof")])
#v(mm-sp.s)
#signature-block(data.signature, theme)
