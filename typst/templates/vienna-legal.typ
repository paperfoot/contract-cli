// ═══════════════════════════════════════════════════════════════════════════
// vienna-legal — Bauhaus-Secession. Terracotta accent. Slab title.
// Heavier hierarchy than helvetica-nera; good for showcase agreements.
// ═══════════════════════════════════════════════════════════════════════════

#import "../shared/contract.typ": data, render-markdown, party-block, signature-block, draft-watermark, clauses-block
#import "../shared/components.typ": mm-sp, sp, lbl, hairline

#let theme = (
  ink:    rgb("#1B1B1B"),
  paper:  rgb("#F5F0E6"),
  accent: rgb("#C74B39"),
  mute:   rgb("#6E685D"),
  hair:   rgb("#C7BFAE"),
  body-font:    ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
  display-font: ("Helvetica Neue", "Helvetica", "Inter", "Arial"),
)

#set page(
  paper: "a4",
  margin: (top: 22mm, bottom: 24mm, left: 22mm, right: 22mm),
  fill: theme.paper,
  background: if data.draft-watermark { draft-watermark(rgb("#E6CFC1")) } else { none },
  footer: align(center,
    text(size: 7.5pt, fill: theme.mute, tracking: 1.2pt)[
      #upper(data.kind-label) · № #data.number · #context counter(page).display() / #context counter(page).final().first()
    ]
  ),
)

#set text(font: theme.body-font, size: 9.8pt, fill: theme.ink, lang: "en")
#set par(leading: 5.6pt, spacing: 5.6pt, justify: true)

// ─── HERO ──
#grid(
  columns: (1fr, auto),
  align: (left + horizon, right + horizon),
  column-gutter: 10mm,
  [
    #if data.logo != none {
      image(data.logo, height: 7.5mm)
      v(3pt)
    }
    #text(size: 8pt, weight: 600, fill: theme.accent, tracking: 1.6pt)[#upper(data.kind-label)]
    #v(2pt)
    #text(size: 26pt, weight: 800, font: theme.display-font, tracking: -1pt)[#data.title]
  ],
  [
    #align(right)[
      #text(size: 9pt, tracking: 2pt, fill: theme.accent, weight: 500)[№ #data.number]
      #v(4pt)
      #lbl(theme, "Effective")
      #v(2pt)
      #text(size: 11pt)[#data.effective-date-display]
    ]
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

#v(mm-sp.m)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.s)

// ─── PREAMBLE ──
#par[
  This #lower(data.kind-label) (this “Agreement”) is made between
  *#data.our-party.legal-name* (the “#data.our-party.role-label”)
  and *#data.their-party.legal-name* (the “#data.their-party.role-label”),
  with effect from #data.effective-date-display, and is governed by the
  terms below.
]

#v(mm-sp.s)
#line(length: 100%, stroke: 0.3pt + theme.hair)
#v(mm-sp.m)

// ─── CLAUSES ──
#clauses-block(data.clauses, theme)

#v(mm-sp.l)
#rect(width: 100%, height: 2pt, fill: theme.accent, stroke: none)
#v(mm-sp.m)

// ─── SIGNATURE ──
#text(size: 11pt, weight: 700, font: theme.display-font, tracking: 0.6pt)[#upper("Signed")]
#v(mm-sp.s)
#signature-block(data.signature, theme)
