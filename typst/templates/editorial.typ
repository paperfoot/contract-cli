// ═══════════════════════════════════════════════════════════════════════════
// editorial — Serif body, narrow measure, generous leading. Suits NDAs and
// elegant short-form agreements you want to read like prose.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, render-markdown, party-block, signature-block, draft-watermark, clauses-block
#import "../shared/components.typ": mm-sp, sp, lbl, hairline

#let theme = (
  ink:    rgb("#181818"),
  paper:  rgb("#FBF9F4"),
  accent: rgb("#2A2A2A"),
  mute:   rgb("#7A7468"),
  hair:   rgb("#D9D2C4"),
  body-font:    ("Georgia", "Newsreader", "EB Garamond", "Times New Roman", "Times"),
  display-font: ("Georgia", "Newsreader", "EB Garamond", "Times New Roman", "Times"),
)

#set page(
  paper: "a4",
  margin: (top: 28mm, bottom: 26mm, left: 26mm, right: 26mm),
  fill: theme.paper,
  background: if data.draft-watermark { draft-watermark(rgb("#E5DCC9")) } else { none },
  footer: align(center,
    text(size: 7.5pt, fill: theme.mute, tracking: 1.2pt, font: theme.body-font)[
      #upper(data.kind-label) · #data.number · #context counter(page).display() of #context counter(page).final().first()
    ]
  ),
)

#set text(font: theme.body-font, size: 10.2pt, fill: theme.ink, lang: "en")
#set par(leading: 6.8pt, spacing: 6.8pt, justify: true, first-line-indent: 0pt)

// ─── HEADER ──
#align(center)[
  #if data.logo != none {
    image(data.logo, height: 7mm)
    v(4pt)
  }
  #text(size: 8pt, tracking: 2.4pt, fill: theme.mute)[#upper(data.kind-label)]
  #v(4pt)
  #text(size: 22pt, font: theme.display-font, weight: 400, style: "italic")[#data.title]
  #v(2pt)
  #text(size: 9.5pt, fill: theme.mute)[Effective #data.effective-date-display · No. #data.number]
]

#v(mm-sp.l)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.m)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

#par[
  This #lower(data.kind-label) is made between
  *#data.our-party.legal-name* (the “#data.our-party.role-label”) and
  *#data.their-party.legal-name* (the “#data.their-party.role-label”),
  with effect from #data.effective-date-display.
]

#v(mm-sp.s)

// ─── CLAUSES ──
#clauses-block(data.clauses, theme)

#v(mm-sp.l)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

#align(center, text(size: 9pt, tracking: 1.6pt, fill: theme.mute)[#upper("In witness whereof")])
#v(mm-sp.s)

#signature-block(data.signature, theme)
