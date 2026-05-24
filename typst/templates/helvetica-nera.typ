// ═══════════════════════════════════════════════════════════════════════════
// helvetica-nera — Sober corporate/legal instrument.
// Left-aligned "instrument panel" header (per Codex's layout suggestion):
// kind + number + effective + law in a 2x2 docket grid up top, then the
// engagement name large left-aligned, then parties as the natural columns.
// White paper, pure black, no accent colour anywhere.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, party-block, clauses-block, signature-block, page-shell, sp, mm-sp, render-markdown

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

// ─── INSTRUMENT PANEL (2-row docket strip) ──
#let panel-cell(lbl-t, value, align-side: left) = [
  #align(align-side)[
    #text(size: 7.5pt, fill: theme.mute, tracking: 1.4pt)[#upper(lbl-t)]\
    #v(1pt)
    #text(size: 9.5pt)[#value]
  ]
]

#grid(
  columns: (1fr, auto),
  row-gutter: 4mm,
  panel-cell("Document", data.kind-label),
  panel-cell("Reference", data.number, align-side: right),
  panel-cell("Effective", data.effective-date-display),
  panel-cell("Governing law", data.governing-law, align-side: right),
)

#v(mm-sp.m)
#line(length: 100%, stroke: 0.6pt + theme.ink)
#v(mm-sp.m)

// ─── TITLE (left-aligned, the engagement name leads) ──
#fit-size(
  (22pt, 20pt, 19pt, 18pt),
  155mm,
  s => text(font: theme.display-font, size: s, weight: 700, tracking: -0.2pt)[#data.title],
)

#v(mm-sp.m)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.m)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.s)

// Compact docket for fee (the other key facts are already in the instrument panel)
#if data.fee-short != none [
  #grid(
    columns: (auto, 1fr),
    column-gutter: 6mm,
    text(size: 7.5pt, fill: theme.mute, tracking: 1.2pt)[#upper("Fee")],
    text(size: 9.5pt)[#data.fee-short]
  )
  #v(mm-sp.s)
  #line(length: 100%, stroke: 0.3pt + theme.hair)
  #v(mm-sp.s)
]

#v(mm-sp.s)

// ─── CLAUSES (no accent colour — full black) ──
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

// ─── SIGNATURE (no shout) ──
#v(mm-sp.l)
#signature-block(data.signature, theme)
