// ═══════════════════════════════════════════════════════════════════════════
// editorial — Centred serif. Generous leading, narrow measure, ceremonial.
// Best for NDAs and short-form elegant agreements.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, star-mark, party-block, meta-strip, clauses-block, signature-block, page-shell, sp, mm-sp

#let theme = (
  ink:         rgb("#181818"),
  paper:       rgb("#FBF9F4"),
  accent:      rgb("#2A2A2A"),
  mute:        rgb("#7A7468"),
  hair:        rgb("#D9D2C4"),
  watermark:   rgb("#E5DCC9"),
  display-font: ("Georgia", "Newsreader", "EB Garamond", "Times New Roman"),
  body-font:    ("Georgia", "Newsreader", "EB Garamond", "Times New Roman"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 26mm, bottom: 26mm, left: 26mm, right: 26mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 10.2pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
)
#set par(leading: 6.8pt, spacing: 6.8pt, justify: true, first-line-indent: 0pt)

// ─── HERO (centred) ──
#align(center)[
  #if data.logo != none {
    image(data.logo, height: 7mm)
  } else {
    star-mark(size: 11pt, color: theme.accent)
  }
  #v(6pt)
  #text(size: 8pt, tracking: 2.4pt, fill: theme.mute)[#upper(data.kind-label)]
  #v(6pt)
  #fit-size(
    (24pt, 22pt, 20pt, 18pt),
    130mm,
    s => text(font: theme.display-font, size: s, weight: 400, style: "italic")[#data.title],
  )
  #v(4pt)
  #text(size: 9.5pt, fill: theme.mute, style: "italic")[Effective #data.effective-date-display · № #data.number]
]

#v(mm-sp.m)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.s)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.m)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.s)

#par[
  This *#lower(data.kind-label)* is made between *#data.our-party.legal-name*
  (the “#data.our-party.role-label”) and *#data.their-party.legal-name*
  (the “#data.their-party.role-label”), with effect from
  #data.effective-date-display.
]

#v(mm-sp.s)
#hairline(theme)
#v(mm-sp.s)

// ─── META STRIP ──
#let meta-pairs = (
  ("Effective", data.effective-date-display),
  ("Term",     data.term-short),
  ("Governing law", data.governing-law),
)
#if data.fee-short != none {
  meta-pairs = meta-pairs + (("Fee", data.fee-short),)
}
#meta-strip(theme, meta-pairs, emphasize-index: 1)

#v(mm-sp.m)

// ─── CLAUSES ──
#clauses-block(data.clauses, theme)

#v(mm-sp.m)
#align(center, line(length: 30mm, stroke: 0.4pt + theme.hair))
#v(mm-sp.m)

#align(center, text(size: 9pt, tracking: 1.6pt, fill: theme.mute, style: "italic")[#upper("In witness whereof")])
#v(mm-sp.s)
#signature-block(data.signature, theme)
