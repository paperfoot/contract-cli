// ═══════════════════════════════════════════════════════════════════════════
// helvetica-nera — Swiss monochrome. Clean, hierarchical, contract-default.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, render-markdown, party-block, signature-block, draft-watermark, clauses-block
#import "../shared/components.typ": mm-sp, sp, lbl, hairline

#let theme = (
  ink:    rgb("#111111"),
  paper:  rgb("#FFFFFF"),
  accent: rgb("#111111"),
  mute:   rgb("#6C6C6C"),
  hair:   rgb("#D6D6D6"),
  body-font:    ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  display-font: ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
)

#set page(
  paper: "a4",
  margin: (top: 22mm, bottom: 22mm, left: 22mm, right: 22mm),
  fill: theme.paper,
  background: if data.draft-watermark { draft-watermark(rgb("#DCDCDC")) } else { none },
  footer: align(center,
    text(size: 7.5pt, fill: theme.mute, tracking: 0.6pt)[
      #upper(data.kind-label) · No. #data.number · Page #context counter(page).display() of #context counter(page).final().first()
    ]
  ),
)

#set text(font: theme.body-font, size: 10pt, fill: theme.ink, lang: "en")
#set par(leading: 5.6pt, spacing: 5.6pt, justify: true)

// ─── HEADER ──
#grid(
  columns: (1fr, auto),
  align: (left + horizon, right + horizon),
  column-gutter: 10mm,
  [
    #if data.logo != none {
      image(data.logo, height: 7mm)
      v(3pt)
    }
    #lbl(theme, data.kind-label)
    #v(2pt)
    #text(size: 18pt, weight: 700, font: theme.display-font, tracking: -0.5pt)[#data.title]
  ],
  [
    #align(right)[
      #lbl(theme, "No.")
      #v(2pt)
      #text(size: 11pt, tracking: 1pt)[#data.number]
      #v(4pt)
      #lbl(theme, "Effective")
      #v(2pt)
      #text(size: 11pt)[#data.effective-date-display]
    ]
  ],
)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.4pt + theme.ink)
#v(mm-sp.m)

// ─── INTRO LINE ──
#par[
  This #lower(data.kind-label) (this “Agreement”) is made between
  *#data.our-party.legal-name* (#data.our-party.role-label)
  and *#data.their-party.legal-name* (#data.their-party.role-label),
  with effect from #data.effective-date-display.
]

#v(mm-sp.s)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.l)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.m)

// ─── CLAUSES ──
#clauses-block(data.clauses, theme)

#v(mm-sp.l)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.l)

// ─── SIGNATURE ──
#text(size: 11pt, weight: 600)[Signatures]
#v(mm-sp.s)
#signature-block(data.signature, theme)
