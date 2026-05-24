// ═══════════════════════════════════════════════════════════════════════════
// vienna-legal — Warm boutique consulting contract.
// Cream paper, terracotta accent. Centred title. No eyebrow.
// Per Codex layout brief: drop eyebrow + "between A and B" subtitle,
// quiet metadata line, smaller terracotta rule, smaller clause numbers.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, party-block, clauses-block, signature-block, page-shell, sp, mm-sp

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
  margin: (top: 22mm, bottom: 20mm, left: 22mm, right: 22mm),
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

// ─── HERO (centred, no eyebrow) ──
#align(center)[
  #if data.logo != none [
    #image(data.logo, height: 7mm)
    #v(8pt)
  ]
  #fit-size(
    (23pt, 21pt, 19pt, 17pt),
    150mm,
    s => text(font: theme.display-font, size: s, weight: 700, tracking: 0pt, fill: theme.ink)[#data.title],
  )
  #v(8pt)
  #text(size: 8.5pt, weight: 500, fill: theme.mute, tracking: 0.2pt)[
    #data.kind-label · #data.number · Effective #data.effective-date-display
  ]
  #v(7pt)
  #rect(width: 32mm, height: 1.4pt, fill: theme.accent, stroke: none)
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

// ─── DOCKET ROW (Term · Law · Fee) ──
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
      #text(size: 7.5pt, fill: theme.mute, tracking: 1pt)[#upper(lbl-t)]\
      #v(1pt)
      #text(size: 9.2pt)[#val]
    ])
  )
]
#line(length: 100%, stroke: 0.3pt + theme.hair)

#v(mm-sp.m)

// ─── CLAUSES (quieter accent numbers) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (7mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.display-font, size: 9.5pt, weight: 700, fill: theme.accent)[#clause.number.],
      [
        #text(font: theme.display-font, size: 10.4pt, weight: 700)[#clause.heading]
        #v(2pt)
        // Inline render of clause markdown via shared helper
        #import "../shared/contract.typ": render-markdown
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE (no shout, no decorative rule) ──
#v(mm-sp.l)
#signature-block(data.signature, theme)
