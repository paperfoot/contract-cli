//! description: Engraved deed — double hairline frame at the masthead, letterspaced Cormorant caps, EB Garamond body with true small caps, witness-style execution. For agreements that should feel notarised.
//! mood: formal, engraved, ceremonial
//! tags: engraved, formal, deed, garamond, small-caps, centred, classic, witness, chancery
//! fonts: Cormorant Garamond, EB Garamond (embedded OFL)
//! paper: ivory
// ═══════════════════════════════════════════════════════════════════════════
// chancery — engraved deed on a centred axis.
// Masthead is a double hairline frame holding the kind-label in letterspaced
// Cormorant Garamond caps. Body is EB Garamond, justified, with REAL small
// caps (smcp) doing the section labels. Execution is witness-style.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, parties-prose-block, signature-block, page-shell, render-markdown, section-label, sp, mm-sp

#let theme = (
  ink:         rgb("#211D18"),
  paper:       rgb("#FBF8F1"),
  accent:      rgb("#211D18"),
  mute:        rgb("#756E62"),
  hair:        rgb("#CFC5B2"),
  watermark:   rgb("#E3DAC6"),
  display-font: ("Cormorant Garamond", "EB Garamond", "Georgia"),
  body-font:    ("EB Garamond", "Georgia", "Times New Roman"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "smallcaps",
  margin: (top: 24mm, bottom: 24mm, left: 28mm, right: 28mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 10.6pt,
  fill: theme.ink,
  lang: "en",
  number-type: "old-style",
  hyphenate: true,
)
#set par(leading: 6.8pt, spacing: 6.8pt, justify: true)

// Real small caps — EB Garamond carries a proper smcp feature.
#let sc-label(txt, size: 10.5pt, tracking: 1.8pt, fill: theme.ink) = text(
  font: theme.body-font, size: size, tracking: tracking, fill: fill,
)[#smallcaps(lower(txt))]

// ─── MASTHEAD: double hairline frame ──
#align(center)[
  #rect(
    width: 100%,
    stroke: 0.5pt + theme.ink,
    inset: 1.8mm,
    rect(
      width: 100%,
      stroke: 0.5pt + theme.ink,
      inset: (x: 8mm, top: 7mm, bottom: 7.5mm),
      align(center)[
        #fit-size(
          (21pt, 19pt, 17pt, 15pt, 13pt),
          120mm,
          s => text(font: theme.display-font, size: s, weight: 600, tracking: 3.5pt)[#upper(data.kind-label)],
        )
        #if data.subtitle != none [
          #v(3.5mm)
          #line(length: 24mm, stroke: 0.4pt + theme.hair)
          #v(3mm)
          #fit-size(
            (12.5pt, 11.5pt, 10.5pt),
            120mm,
            s => text(font: theme.body-font, size: s, style: "italic", fill: theme.mute)[#data.subtitle],
          )
        ]
      ],
    ),
  )
]

#v(mm-sp.m)

// ─── DATED (centred, ceremonial) ──
#align(center, text(size: 10.6pt)[#smallcaps[dated] #h(4pt) #data.effective-date-display])

#v(mm-sp.m)

// ─── PARTIES ──
#align(center, sc-label("This Agreement is made between"))
#v(2pt)
#parties-prose-block(data.parties-prose, theme)

#v(mm-sp.s)
#align(center, line(length: 24mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.s)

// ─── KEY TERMS (centred, small-caps labels) ──
#let cells = (("Term", data.term-short), ("Governing law", data.governing-law))
#if data.fee-short != none {
  cells = cells + (("Fee", data.fee-short),)
}
#align(center, grid(
  columns: cells.map(_ => auto),
  column-gutter: 14mm,
  align: (center + top, center + top, center + top),
  ..cells.map(((lbl-t, val)) => [
    #sc-label(lbl-t, size: 8.5pt, tracking: 1.4pt, fill: theme.mute)\
    #v(1pt)
    #text(size: 10.2pt)[#val]
  ])
))

#v(mm-sp.s)
#align(center, line(length: 24mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

// ─── AGREED TERMS ──
#align(center, sc-label("Agreed Terms", tracking: 2.2pt))
#v(mm-sp.xs)

// ─── CLAUSES (small-caps headings on the centred axis's left rail) ──
#for clause in data.clauses {
  block(breakable: true, spacing: mm-sp.s, [
    #grid(
      columns: (9mm, 1fr),
      column-gutter: 3.5mm,
      align: (right + top, left + top),
      text(font: theme.body-font, size: 11pt, weight: 500)[#clause.number.],
      [
        #text(font: theme.body-font, size: 11pt, weight: 500)[#smallcaps(lower(clause.heading))]
        #v(2pt)
        #render-markdown(clause.body)
      ],
    )
  ])
}

// ─── EXECUTION (witness-style) ──
#v(mm-sp.l)
#block(breakable: false, [
  #align(center, line(length: 24mm, stroke: 0.4pt + theme.hair))
  #v(mm-sp.s)
  #align(center, sc-label("In witness whereof", tracking: 2.2pt))
  #v(2pt)
  #align(center, text(size: 10.2pt, style: "italic", fill: theme.mute)[
    the parties have executed this agreement on the date first written above.
  ])
  #v(mm-sp.s)
  #signature-block(data.signature, theme)
])
