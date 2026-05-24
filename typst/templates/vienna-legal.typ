// ═══════════════════════════════════════════════════════════════════════════
// vienna-legal — Bauhaus-Secession. Terracotta accent, slab title, lined
// numerals, fit-size title. Polished centrepiece template; companion to
// invoice-cli's `vienna` template.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, lbl, hairline, fit-size, star-mark, party-block, meta-strip, clauses-block, signature-block, page-shell, sp, mm-sp

#let theme = (
  ink:         rgb("#1B1B1B"),
  paper:       rgb("#F5F0E6"),
  accent:      rgb("#C74B39"),
  accent-soft: rgb("#EDE6D6"),
  mute:        rgb("#6E685D"),
  hair:        rgb("#C7BFAE"),
  watermark:   rgb("#E6CFC1"),
  display-font: ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  body-font:    ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
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
        image(data.logo, height: 8mm)
      } else {
        star-mark(size: 13pt, color: theme.accent)
      },
      text(size: 8.5pt, weight: 600, fill: theme.accent, tracking: 1.8pt)[#upper(data.kind-label)],
    )
    #v(2pt)
    #fit-size(
      (28pt, 24pt, 22pt, 20pt, 18pt),
      105mm,
      s => text(font: theme.display-font, size: s, weight: 800, tracking: -1pt, fill: theme.ink)[#data.title],
    )
  ],
  [
    #fit-size(
      (10pt, 9.5pt, 9pt),
      75mm,
      s => text(size: s, tracking: 2pt, fill: theme.accent, weight: 500)[№ #data.number],
    )
    #v(4pt)
    #lbl(theme, "Effective")
    #v(2pt)
    #text(size: 11pt, weight: 600)[#data.effective-date-display]
  ],
)

#v(mm-sp.s)
#rect(width: 100%, height: 3pt, fill: theme.accent, stroke: none)
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
  This *#lower(data.kind-label)* (this “Agreement”) is made between
  *#data.our-party.legal-name* (the “#data.our-party.role-label”) and
  *#data.their-party.legal-name* (the “#data.their-party.role-label”),
  with effect from #data.effective-date-display. It is governed by the terms below.
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
#rect(width: 100%, height: 2pt, fill: theme.accent, stroke: none)
#v(mm-sp.s)

// ─── SIGNATURE ──
#text(font: theme.display-font, size: 11pt, weight: 700, tracking: 0.6pt)[#upper("Signed")]
#v(mm-sp.s)
#signature-block(data.signature, theme)
