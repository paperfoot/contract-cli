// ═══════════════════════════════════════════════════════════════════════════
// helvetica-nera — Swiss monochrome. Clean, hierarchical. Default template.
// Mirrors invoice-cli's `helvetica-nera` aesthetic.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, star-mark, party-block, meta-strip, clauses-block, signature-block, page-shell, sp, mm-sp

#let theme = (
  ink:         rgb("#111111"),
  paper:       rgb("#FFFFFF"),
  accent:      rgb("#111111"),
  mute:        rgb("#6C6C6C"),
  hair:        rgb("#D6D6D6"),
  watermark:   rgb("#D8D8D8"),
  display-font: ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  body-font:    ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  mono-font:    ("Menlo", "DejaVu Sans Mono"),
  label-style: "upper",
  margin: (top: 22mm, bottom: 22mm, left: 22mm, right: 22mm),
)

#show: body => page-shell(theme, body)

#set text(
  font: theme.body-font,
  size: 9.8pt,
  fill: theme.ink,
  lang: "en",
  number-type: "lining",
  number-width: "tabular",
)
#set par(leading: 5.8pt, spacing: 5.8pt, justify: true)

// ─── HERO ──
#grid(
  columns: (1fr, auto),
  align: (left + horizon, right + horizon),
  column-gutter: 10mm,
  [
    #grid(
      columns: (auto, 1fr),
      column-gutter: 8pt,
      align: (horizon, horizon),
      if data.logo != none {
        image(data.logo, height: 7mm)
      } else {
        star-mark(size: 12pt, color: theme.ink)
      },
      text(size: 8.5pt, weight: 600, fill: theme.mute, tracking: 1.6pt)[#upper(data.kind-label)],
    )
    #v(2pt)
    #fit-size(
      (22pt, 20pt, 18pt, 17pt, 16pt),
      110mm,
      s => text(font: theme.display-font, size: s, weight: 700, tracking: -0.4pt)[#data.title],
    )
  ],
  [
    #lbl(theme, "No.")
    #v(2pt)
    #text(size: 11pt, tracking: 0.8pt)[#data.number]
    #v(4pt)
    #lbl(theme, "Effective")
    #v(2pt)
    #text(size: 11pt, weight: 600)[#data.effective-date-display]
  ],
)

#v(mm-sp.s)
#line(length: 100%, stroke: 0.6pt + theme.ink)
#v(mm-sp.m)

// ─── PARTIES ──
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14mm,
  party-block(data.our-party, theme),
  party-block(data.their-party, theme),
)

#v(mm-sp.s)
#hairline(theme)
#v(mm-sp.s)

// ─── PREAMBLE ──
#par[
  This *#lower(data.kind-label)* (the “Agreement”) is made between
  *#data.our-party.legal-name* (the “#data.our-party.role-label”)
  and *#data.their-party.legal-name* (the “#data.their-party.role-label”),
  with effect from #data.effective-date-display.
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
#hairline(theme, weight: 0.6pt)
#v(mm-sp.s)

// ─── SIGNATURE ──
#text(font: theme.display-font, size: 11pt, weight: 700)[Signatures]
#v(mm-sp.s)
#signature-block(data.signature, theme)
