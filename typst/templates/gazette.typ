//! description: Front-page broadsheet — Fraunces Black masthead over thick-and-thin rules, newspaper dateline folio, ragged Newsreader body. A contract set like page one.
//! mood: literary, statement, editorial
//! tags: magazine, masthead, editorial, serif, white, ragged, statement, broadsheet, newspaper
//! fonts: Fraunces 72pt, Newsreader 16pt, Libre Franklin (embedded OFL)
//! paper: white
// ═══════════════════════════════════════════════════════════════════════════
// gazette — magazine masthead / front-page broadsheet.
// The kind-label is set as a newspaper masthead in Fraunces Black across
// the full measure, over a thick-over-thin double rule. A dateline "folio"
// strip (No. · Dated · Governing law) runs beneath in Libre Franklin caps.
// Body is Newsreader, ragged right — literary, no drop caps, no tricks.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#141414"),
  paper:       rgb("#FFFFFF"),
  accent:      rgb("#141414"),
  mute:        rgb("#5F5F5F"),
  hair:        rgb("#D9D9D9"),
  watermark:   rgb("#DCDCDC"),
  display-font: ("Libre Franklin", "Helvetica Neue", "Helvetica", "Arial"),
  body-font:    ("Newsreader 16pt", "Georgia", "Times New Roman"),
  masthead-font: ("Fraunces 72pt", "Georgia", "Times New Roman"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 20mm, bottom: 22mm, left: 22mm, right: 22mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 9.8pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  hyphenate: false,
)
#set par(leading: 6pt, spacing: 6pt, justify: false)

// ─── MASTHEAD (Fraunces Black, full measure) ──
#align(center)[
  #fit-size(
    (34pt, 31pt, 28pt, 25pt, 22pt, 19pt),
    162mm,
    s => text(font: theme.masthead-font, size: s, weight: 900, tracking: -0.3pt)[#data.kind-label],
  )
]

#v(2.4mm)
// Thick-over-thin double rule — the broadsheet signature.
#line(length: 100%, stroke: 2.2pt + theme.ink)
#v(1.1mm)
#line(length: 100%, stroke: 0.5pt + theme.ink)
#v(1.6mm)

// ─── DATELINE FOLIO (No. · Dated · Governing law) ──
#grid(
  columns: (1fr, auto, 1fr),
  align: (left + horizon, center + horizon, right + horizon),
  text(font: theme.display-font, size: 7pt, weight: 600, tracking: 1pt, fill: theme.ink)[NO. #upper(data.number)],
  text(font: theme.display-font, size: 7pt, weight: 600, tracking: 1pt, fill: theme.ink)[DATED #upper(data.effective-date-display)],
  text(font: theme.display-font, size: 7pt, weight: 600, tracking: 1pt, fill: theme.ink)[#upper(data.governing-law) LAW],
)
#v(1.6mm)
#line(length: 100%, stroke: 0.5pt + theme.ink)

// ─── DECK (project name as the standfirst) ──
#if data.subtitle != none [
  #v(4mm)
  #align(center)[
    #fit-size(
      (14pt, 13pt, 12pt, 11pt),
      150mm,
      s => text(font: theme.body-font, size: s, style: "italic", fill: theme.ink)[#data.subtitle],
    )
  ]
]

#v(mm-sp.m)

// ─── PARTIES ──
#text(font: theme.display-font, size: 8pt, weight: 600, tracking: 1.4pt)[PARTIES]
#v(2pt)
#parties-prose-block(data.parties-prose, theme)

#v(mm-sp.s)
#hairline(theme)
#v(mm-sp.s)

// ─── KEY TERMS ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#grid(
  columns: cells.map(_ => (100% - 16mm) / 3),
  column-gutter: 8mm,
  align: (left + top, left + top, left + top),
  ..cells.map(((lbl-t, val)) => [
    #text(font: theme.display-font, size: 7pt, fill: theme.mute, weight: 600, tracking: 1.2pt)[#upper(lbl-t)]\
    #v(1pt)
    #text(size: 9.8pt)[#val]
  ])
)

#v(mm-sp.s)
#hairline(theme)
#v(mm-sp.m)

// ─── AGREED TERMS ──
#text(font: theme.display-font, size: 8pt, weight: 600, tracking: 1.4pt)[AGREED TERMS]
#v(mm-sp.xs)

// ─── CLAUSES (literary: serif headline, hanging number) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (8mm, 1fr),
      column-gutter: 4mm,
      align: (right + top, left + top),
      text(font: theme.body-font, size: 11pt, weight: 600, fill: theme.mute)[#clause.number.],
      [
        #text(font: theme.body-font, size: 11pt, weight: 600)[#clause.heading]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── SIGNATURE (closing thin-over-thick rule travels with the block) ──
#v(mm-sp.l)
#block(breakable: false, [
  #line(length: 100%, stroke: 0.5pt + theme.ink)
  #v(0.9mm)
  #line(length: 100%, stroke: 2.2pt + theme.ink)
  #v(mm-sp.s)
  #signature-block(data.signature, theme)
])
