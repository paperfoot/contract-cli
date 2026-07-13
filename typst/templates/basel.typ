//! description: Swiss brutalist instrument — full-width black bar, 42mm apparatus rail, oversized clause numerals, ragged Helvetica body. Severe and architectural.
//! mood: severe, modernist, architectural
//! tags: swiss, brutalist, grid, monochrome, rail, severe, modernist, architectural
//! fonts: Helvetica Neue (system), Archivo (embedded OFL)
//! paper: white
// ═══════════════════════════════════════════════════════════════════════════
// basel — Swiss brutalist asymmetric grid.
// A 42mm left rail owns ALL apparatus: DATED, the parties summary, and
// TERM/LAW/FEE stacked as Archivo marginalia. The body runs ragged in
// Helvetica Neue on the right. Clause numerals sit oversized in the rail.
// One full-width black bar at the top; everything else is hairlines.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#0B0B0B"),
  paper:       rgb("#FFFFFF"),
  accent:      rgb("#0B0B0B"),
  mute:        rgb("#5A5A5A"),
  hair:        rgb("#C9C9C9"),
  watermark:   rgb("#DCDCDC"),
  display-font: ("Archivo", "Helvetica Neue", "Helvetica", "Arial"),
  body-font:    ("Helvetica Neue", "Archivo", "Helvetica", "Arial"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 18mm, bottom: 22mm, left: 18mm, right: 18mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 9.4pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  number-width: "tabular",
  hyphenate: false,
)
#set par(leading: 5.6pt, spacing: 5.6pt, justify: false)

// The one grid geometry everything obeys: 42mm rail · 8mm gutter · body.
#let rail-row(rail, body) = grid(
  columns: (42mm, 1fr),
  column-gutter: 8mm,
  align: (left + top, left + top),
  rail, body,
)

#let rail-label(txt) = text(
  font: theme.display-font, size: 6.5pt, weight: 600,
  tracking: 1.4pt, fill: theme.mute,
)[#upper(txt)]

#let rail-value(txt) = text(
  font: theme.display-font, size: 8.6pt, weight: 500, fill: theme.ink,
)[#txt]

#let rail-rule = line(length: 100%, stroke: 0.5pt + theme.ink)

// ─── FULL-WIDTH BLACK BAR ──
#rect(width: 100%, height: 7mm, fill: theme.ink)

#v(mm-sp.m)

// ─── HEAD ROW: apparatus rail · title + parties ──
#rail-row(
  [
    // DATED
    #rail-rule
    #v(sp.s)
    #rail-label("Dated")
    #v(sp.xxs)
    #rail-value(data.effective-date-display)
    #v(sp.m)
    // PARTIES summary — display names only; the instrument text carries
    // the full legal recitals on the right.
    #rail-rule
    #v(sp.s)
    #rail-label(data.signature.our-label)
    #v(sp.xxs)
    #rail-value(data.our-party.display-name)
    #v(sp.s)
    #rail-label(data.signature.their-label)
    #v(sp.xxs)
    #rail-value(data.their-party.display-name)
    #v(sp.m)
    // TERM / LAW / FEE marginalia
    #rail-rule
    #v(sp.s)
    #rail-label("Term")
    #v(sp.xxs)
    #rail-value(data.term-short)
    #v(sp.s)
    #rail-label("Governing law")
    #v(sp.xxs)
    #rail-value(data.governing-law)
    #if data.fee-short != none [
      #v(sp.s)
      #rail-label("Fee")
      #v(sp.xxs)
      #rail-value(data.fee-short)
    ]
  ],
  [
    // Title — Helvetica, heavy, tight.
    #fit-size(
      (25pt, 23pt, 21pt, 19pt, 17pt),
      120mm,
      s => text(font: theme.body-font, size: s, weight: 700, tracking: -0.3pt)[#data.kind-label],
    )
    #if data.subtitle != none [
      #v(3pt)
      #text(font: theme.body-font, size: 11.5pt, weight: 400, fill: theme.mute)[#data.subtitle]
    ]
    #v(mm-sp.m)
    #text(font: theme.display-font, size: 7pt, weight: 600, tracking: 1.4pt, fill: theme.mute)[PARTIES]
    #v(2pt)
    #parties-prose-block(data.parties-prose, theme)
  ],
)

#v(mm-sp.m)

// AGREED TERMS sits in its own row so it hugs clause 1 even when the
// apparatus rail is taller than the head-row body.
#rail-row([], text(font: theme.display-font, size: 7pt, weight: 600, tracking: 1.4pt, fill: theme.mute)[AGREED TERMS])

#v(mm-sp.xs)

// ─── CLAUSES — oversized numeral in the rail, body right ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.m, rail-row(
    align(right, text(
      font: theme.display-font, size: 21pt, weight: 600,
      tracking: -0.3pt, fill: theme.ink,
    )[#clause.number]),
    [
      #v(2.4pt) // optically align heading baseline with the numeral
      #text(font: theme.body-font, size: 10.5pt, weight: 700)[#clause.heading]
      #v(2pt)
      #render-markdown(clause.body)
    ],
  ))
}

// ─── EXECUTION ──
#v(mm-sp.l)
#block(breakable: false, rail-row(
  [
    #rail-rule
    #v(sp.s)
    #rail-label("Execution")
  ],
  signature-block(data.signature, theme),
))
