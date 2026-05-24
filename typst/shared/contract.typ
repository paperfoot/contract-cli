// ═══════════════════════════════════════════════════════════════════════════
// contract-cli — shared helpers used by every template.
//
// Loads the Rust-generated `contract.json` sidecar and exposes:
//   - `data` (the full ContractRenderData payload)
//   - typographic helpers (lbl, hairline, fit-size, star-mark)
//   - structural components (party-block, meta-strip, clauses-block,
//     signature-block, draft-watermark)
//   - `page-shell` — wraps the body with consistent page chrome (compact
//     header on pages 2+, pagination footer)
//
// Components handle STRUCTURE, OVERFLOW, PAGINATION, SPACING.
// Templates own AESTHETIC (palette, fonts, hero composition).
// ═══════════════════════════════════════════════════════════════════════════

#let data = json("contract.json")

// ─── Spacing scale ────────────────────────────────────────────────────────

#let sp = (
  xxs: 2pt,
  xs:  4pt,
  s:   6pt,
  m:   10pt,
  l:   16pt,
  xl:  24pt,
)

#let mm-sp = (
  xs: 3mm,
  s:  5mm,
  m:  8mm,
  l:  12mm,
  xl: 16mm,
)

// ─── Theme helper ──────────────────────────────────────────────────────────

#let th(theme, key, default) = if key in theme { theme.at(key) } else { default }

// ─── Dynamic text-size fitting via measure() ───────────────────────────────
// Pick the largest size from `sizes` whose rendered width fits `max-width`.

#let fit-size(sizes, max-width, make) = context {
  let chosen = sizes.last()
  for s in sizes {
    if measure(make(s)).width <= max-width {
      chosen = s
      break
    }
  }
  make(chosen)
}

// ─── Star-mark (logo fallback) ─────────────────────────────────────────────

#let star-mark(size: 14pt, color: black) = {
  let s = size
  box(width: s, height: s, {
    place(top + left, polygon(
      fill: color,
      (s * 0.5, s * 0.0),
      (s * 0.58, s * 0.42),
      (s * 1.0, s * 0.5),
      (s * 0.58, s * 0.58),
      (s * 0.5, s * 1.0),
      (s * 0.42, s * 0.58),
      (s * 0.0, s * 0.5),
      (s * 0.42, s * 0.42),
    ))
  })
}

// ─── Labels respect theme.label-style ──────────────────────────────────────

#let lbl(theme, txt, size: 7.5pt, tracking: 1.2pt, fill: none, weight: 500) = {
  let style = th(theme, "label-style", "upper")
  let f = if fill == none { th(theme, "mute", rgb("#666666")) } else { fill }
  if style == "smallcaps" {
    text(
      font: th(theme, "display-font", ("Helvetica Neue", "Helvetica", "Arial")),
      size: size + 0.3pt, tracking: tracking + 0.2pt, fill: f, weight: weight,
    )[#upper(txt)]
  } else if style == "mono-tag" {
    text(
      font: th(theme, "mono-font", ("Menlo", "DejaVu Sans Mono")),
      size: size - 0.5pt, tracking: tracking - 0.3pt, fill: f, weight: weight,
    )[#upper(txt)]
  } else {
    text(size: size, tracking: tracking, fill: f, weight: weight)[#upper(txt)]
  }
}

#let hairline(theme, weight: 0.3pt) = {
  let hair = th(theme, "hair", rgb("#dadada"))
  line(length: 100%, stroke: weight + hair)
}

// ─── Minimal markdown → Typst rendering ────────────────────────────────────
// Paragraph breaks on blank lines; bullet lists from "- " lines; numbered
// lists from "1. " / "12. " lines. Anything else: flowing paragraph.

#let starts-with-number(s) = {
  let m = s.match(regex("^\d+\.\s"))
  m != none
}

#let strip-num-prefix(s) = {
  let m = s.match(regex("^\d+\.\s+"))
  if m == none { s } else { s.slice(m.end) }
}

#let render-markdown(md) = {
  let paras = md.split("\n\n").map(p => p.trim())
  for p in paras {
    if p == "" { continue }
    let lines = p.split("\n")
    let first = lines.find(l => l.trim() != "")
    if first == none { continue }
    let first-t = first.trim()
    if first-t.starts-with("- ") {
      list(
        ..lines
          .filter(l => l.trim().starts-with("- "))
          .map(l => l.trim().slice(2))
      )
    } else if starts-with-number(first-t) {
      enum(
        ..lines
          .filter(l => starts-with-number(l.trim()))
          .map(l => strip-num-prefix(l.trim()))
      )
    } else {
      let joined = lines.map(l => l.trim()).filter(l => l != "").join(" ")
      par(joined)
    }
  }
}

// ─── Party block ───────────────────────────────────────────────────────────
// Mirrors invoice-cli's hierarchy: role label (tracked) → display name (bold)
// → legal name (italic mute, if different) → attn → address lines →
// id strip (Co. <company-no> · <jurisdiction>).
// Strict 3-size scale: 11pt heading, 9.5pt body, 7.5pt labels.

#let party-block(party, theme) = {
  let mute = th(theme, "mute", rgb("#666666"))
  let display = th(theme, "display-font", ("Helvetica Neue", "Helvetica", "Arial"))
  let has(k) = k in party and party.at(k) != none and party.at(k) != ""

  lbl(theme, party.role-label)
  v(sp.s)
  text(font: display, size: 11pt, weight: 600)[#party.display-name]
  linebreak()
  if has("legal-name") and party.legal-name != party.display-name {
    text(size: 9.5pt, fill: mute, style: "italic")[#party.legal-name]
    linebreak()
  }
  if has("attn") {
    text(size: 9.5pt, fill: mute)[Attn: #party.attn]
    linebreak()
  }
  for line in party.address [#text(size: 9.5pt)[#line]\ ]
  let id-bits = ()
  if has("company-no") { id-bits.push("Co. " + party.company-no) }
  if has("jurisdiction") { id-bits.push(party.jurisdiction) }
  if id-bits.len() > 0 {
    text(size: 9.5pt, fill: mute)[#id-bits.join(" · ")]
    linebreak()
  }
  if has("email") {
    text(size: 9pt, fill: mute)[#party.email]
  }
}

// ─── Meta strip ───────────────────────────────────────────────────────────
// One-row strip of `(label, value)` pairs. The value at `emphasize-index`
// is rendered in accent colour. Used for the "Effective | Term | Law" strip.

#let meta-cell(theme, lbl-text, value, emphasize: false) = [
  #lbl(theme, lbl-text)
  #v(sp.xs)
  #if emphasize {
    text(size: 9.5pt, weight: 600, fill: th(theme, "accent", black))[#value]
  } else {
    text(size: 9.5pt)[#value]
  }
]

#let meta-strip(theme, pairs, emphasize-index: -1) = {
  let cols = pairs.map(_ => auto)
  // Make the last cell flex so a long governing-law string doesn't crowd
  // the others. If only 4 cells, give col 2 (Term) the 1fr.
  if pairs.len() >= 3 {
    cols = ()
    for i in range(pairs.len()) {
      if i == pairs.len() - 2 { cols.push(1fr) } else { cols.push(auto) }
    }
  }
  grid(
    columns: cols,
    column-gutter: mm-sp.s,
    align: (left + top, left + top, left + top, left + top, left + top, left + top),
    ..pairs.enumerate().map(((i, p)) => meta-cell(theme, p.at(0), p.at(1), emphasize: i == emphasize-index))
  )
}

// ─── Clauses block ─────────────────────────────────────────────────────────
// Numbered clauses with accent number, bold heading, body via render-markdown.
// `breakable: true` lets long clauses split across pages cleanly.

#let clauses-block(clauses, theme) = {
  let accent = th(theme, "accent", rgb("#333333"))
  let display = th(theme, "display-font", ("Helvetica Neue", "Helvetica", "Arial"))
  for clause in clauses {
    block(breakable: true, spacing: mm-sp.s, [
      #grid(
        columns: (10mm, 1fr),
        column-gutter: 4mm,
        align: (right + top, left + top),
        text(font: display, size: 11pt, weight: 600, fill: accent)[#clause.number.],
        [
          #text(font: display, size: 11pt, weight: 600)[#clause.heading]
          #v(sp.xs)
          #render-markdown(clause.body)
        ],
      )
    ])
  }
}

// ─── Signature block ───────────────────────────────────────────────────────
// Two columns. Each side: small tracked "Signed for and on behalf of" label,
// party legal name, signature line, name/title/date rows.

#let _sig-row(theme, label, value) = {
  let mute = th(theme, "mute", rgb("#666666"))
  grid(
    columns: (12mm, 1fr),
    column-gutter: 2mm,
    text(size: 7.5pt, fill: mute, tracking: 0.8pt)[#upper(label)],
    text(size: 9.5pt)[#if value != none { value } else { "" }],
  )
}

#let signature-pair(theme, party-label, party-name, signer-name, signer-title, signer-date) = {
  lbl(theme, "Signed for and on behalf of")
  v(sp.xs)
  text(font: th(theme, "display-font", ("Helvetica Neue", "Helvetica", "Arial")), size: 10.5pt, weight: 600)[#party-name]
  v(18mm)
  line(length: 100%, stroke: 0.4pt + th(theme, "ink", black))
  v(sp.xs)
  _sig-row(theme, "Name", signer-name)
  v(sp.xxs)
  _sig-row(theme, "Title", signer-title)
  v(sp.xxs)
  _sig-row(theme, "Date", signer-date)
}

#let signature-block(sig, theme) = {
  // breakable: false — execution block must NEVER split across pages.
  // If there isn't enough room left on the current page, Typst pushes the
  // whole signature block to the next page.
  block(breakable: false, [
    #grid(
      columns: (1fr, 1fr),
      column-gutter: mm-sp.l,
      signature-pair(
        theme,
        sig.our-label,
        sig.our-name,
        sig.at("our-signer-name", default: none),
        sig.at("our-signer-title", default: none),
        sig.at("our-signer-date", default: none),
      ),
      signature-pair(
        theme,
        sig.their-label,
        sig.their-name,
        sig.at("their-signer-name", default: none),
        sig.at("their-signer-title", default: none),
        sig.at("their-signer-date", default: none),
      ),
    )
  ])
}

// ─── Parties prose block ──────────────────────────────────────────────────
// Renders `data.parties-prose` as the traditional UK numbered (1)/(2) list,
// with "; and" between parties and a full stop after the last one.

#let parties-prose-block(prose-lines, theme) = {
  block(breakable: false, [
    #for (i, line) in prose-lines.enumerate() [
      #v(3pt)
      #grid(
        columns: (10mm, 1fr),
        column-gutter: 2mm,
        align: (right + top, left + top),
        text(size: 9.6pt, weight: 500)[(#str(i + 1))],
        // The Rust side emits `*NAME*` for the bold party name — eval the
        // whole string as markup so Typst interprets the bold marker.
        eval(line + if i == prose-lines.len() - 1 { "." } else { "; and" }, mode: "markup"),
      )
    ]
  ])
}

// ─── Draft watermark ───────────────────────────────────────────────────────

#let draft-watermark(color) = {
  place(
    center + horizon,
    rotate(
      -28deg,
      text(size: 110pt, weight: 700, fill: color, tracking: 6pt)[DRAFT],
    ),
  )
}

// ─── Section label helper (DATED / PARTIES / AGREED TERMS) ─────────────────
// Single point of truth — call from each template so labels look identical.

#let section-label(theme, txt, size: 9pt, weight: 600, tracking: 0.6pt) = {
  text(font: th(theme, "display-font", ("Helvetica Neue",)),
       size: size, weight: weight, tracking: tracking)[#upper(txt)]
}

// ─── Compact page-2+ header strip ─────────────────────────────────────────
// Carries the document type + reference. Page count lives in the FOOTER —
// avoiding double-stamping the same info top and bottom of the page.

#let compact-strip(theme) = {
  let mute = th(theme, "mute", rgb("#666666"))
  pad(top: mm-sp.s, bottom: 0mm)[
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      context fit-size(
        (8pt, 7.5pt, 7pt),
        160mm,
        s => text(size: s, fill: mute, tracking: 0.3pt)[#upper(data.kind-label) · No. #data.number],
      ),
      [],
    )
    #v(sp.xs)
    #hairline(theme)
  ]
}

// ─── Pagination footer ────────────────────────────────────────────────────

#let pagination-footer(theme) = {
  let mute = th(theme, "mute", rgb("#666666"))
  pad(top: 0mm, bottom: mm-sp.s)[
    #hairline(theme)
    #v(sp.xs)
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      // Internal reference code lives only in the footer, at small size,
      // so it doesn't intrude on the body. Useful for filing, invisible
      // on a casual read.
      text(size: 6.5pt, fill: mute, tracking: 0.4pt)[Ref. #data.number],
      context text(size: 6.5pt, fill: mute, tracking: 0.4pt)[
        Page #here().page() of #counter(page).final().first()
      ],
    )
  ]
}

// ─── Page shell ────────────────────────────────────────────────────────────

#let page-shell(theme, body) = {
  set page(
    paper: "a4",
    margin: th(theme, "margin", (top: 22mm, bottom: 22mm, left: 22mm, right: 22mm)),
    fill: th(theme, "paper", white),
    header: context if here().page() > 1 { compact-strip(theme) },
    footer: pagination-footer(theme),
    background: if data.draft-watermark {
      draft-watermark(th(theme, "watermark", rgb("#DCDCDC")))
    } else { none },
  )
  body
}
