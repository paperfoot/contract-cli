// ═══════════════════════════════════════════════════════════════════════════
// editorial — Formal serif. Centred title. Roman semibold (italic only on
// the project subtitle when present). Reads like a deed.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

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

// ─── HERO ──
#align(center)[
  #fit-size(
    (26pt, 24pt, 22pt, 20pt),
    132mm,
    s => text(font: theme.display-font, size: s, weight: 600)[#data.kind-label],
  )
  #if data.subtitle != none [
    #v(8pt)
    #fit-size(
      (13pt, 12pt, 11pt),
      132mm,
      s => text(font: theme.display-font, size: s, weight: 400, style: "italic", fill: theme.mute)[#data.subtitle],
    )
  ]
  #v(10pt)
  #line(length: 28mm, stroke: 0.4pt + theme.hair)
]

#v(mm-sp.m)

// ─── DATED ──
#text(size: 10pt, style: "italic")[Dated #data.effective-date-display.]

#v(mm-sp.s)

// ─── PARTIES ──
#section-label(theme, "Parties", size: 10pt, tracking: 0.8pt)
#v(2pt)
#parties-prose-block(data.parties-prose, theme)

#v(mm-sp.s)
#align(center, line(length: 28mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.s)

// ─── KEY TERMS (italic labels) ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#grid(
  columns: cells.map(_ => 1fr),
  column-gutter: 8mm,
  align: (left + horizon, left + horizon, left + horizon),
  ..cells.map(((lbl-t, val)) => [
    #text(size: 7.5pt, fill: theme.mute, tracking: 1pt, style: "italic")[#upper(lbl-t)]\
    #v(1pt)
    #text(size: 9.8pt)[#val]
  ])
)

#v(mm-sp.s)
#align(center, line(length: 28mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

// ─── AGREED TERMS ──
#section-label(theme, "Agreed terms", size: 10pt, tracking: 0.8pt)
#v(mm-sp.xs)

// ─── CLAUSES ──
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
