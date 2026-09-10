#import "contract.typ": data, page-shell, hairline, render-markdown, th

// One reading grid and type scale for the entire collection. Template themes
// supply font character and colour; they cannot change the document geometry.
#let gutter = 8mm
#let quiet(theme, body) = text(size: 9.5pt, fill: theme.mute, body)

#let party-details(party, theme) = {
  set par(leading: 4pt, spacing: 7pt)
  quiet(theme, party.role-label)
  v(5pt)
  text(weight: 600)[#if party.legal-name != none { party.legal-name } else { party.display-name }]
  v(5pt)
  text(size: 9.5pt)[
    #party.address.join(", ")
    #if party.company-no != none { linebreak(); [Company no. #party.company-no] }
    #if party.email != none { linebreak(); party.email }
  ]
}

#let execution(theme) = {
  let sig = data.signature
  let field(label, value) = {
    grid(columns: (13mm, 1fr), column-gutter: 2mm, align: bottom,
      quiet(theme, label),
      if value != none and value != "" {
        text(size: 10.5pt, value)
      } else {
        block(height: 9mm, width: 100%)[
          #place(bottom, line(length: 100%, stroke: 0.35pt + theme.hair))
        ]
      },
    )
  }
  let party(prefix) = {
    let get(key) = sig.at(prefix + key, default: none)
    set par(leading: 4pt, spacing: 7pt)
    quiet(theme, "Signed by / for")
    v(5pt)
    text(weight: 600)[#get("-name")]
    v(15mm)
    line(length: 100%, stroke: 0.45pt + theme.mute)
    v(4pt)
    quiet(theme, "Signature")
    v(5pt)
    field("Name", get("-signer-name"))
    field("Title", get("-signer-title"))
    field("Date", get("-signer-date"))
  }
  block(breakable: false, above: 24pt)[
    #text(size: th(theme, "body-size", 11.25pt) + 1pt, weight: 600)[Agreement and signatures]
    #v(8pt)
    #text(size: 10.5pt)[The parties agree to the terms set out above.]
    #v(18pt)
    #grid(columns: (1fr, 1fr), column-gutter: 12mm, party("our"), party("their"))
  ]
}

#let modern-contract(theme, character: "folio") = {
  let body-size = th(theme, "body-size", 11.25pt)
  set text(font: theme.body-font, size: body-size, fill: theme.ink,
    lang: "en", hyphenate: false, number-type: "lining")
  // Both values measure ink-to-ink gaps. Six extra points between paragraphs
  // create separation without the contradictory use of first-line indents.
  let line-gap = if theme.body-font == "Literata" { 6.5pt } else { 5.5pt }
  set par(leading: line-gap, spacing: line-gap + 6pt, justify: false, first-line-indent: 0pt)
  set list(indent: 1mm, body-indent: 3mm, spacing: 8pt)
  set enum(indent: 1mm, body-indent: 3mm, spacing: 8pt)
  set heading(numbering: none)
  show heading.where(level: 1): it => block(above: 0pt, below: 0pt, sticky: true)[
    #set par(leading: 5pt)
    #text(font: theme.display-font, size: 27pt,
      weight: th(theme, "title-weight", 500), tracking: -0.35pt)[#it.body]
  ]
  show heading.where(level: 2): it => block(above: 19pt, below: 7pt, sticky: true)[
    #text(size: body-size + 1pt, weight: 600)[#it.body]
  ]
  // Fixed margins also apply to Letter. The number gutter is outside the
  // reading column, so one- and two-digit clauses share an identical axis.
  let theme = theme + (margin: (top: 25mm, bottom: 25mm, left: 30mm, right: 30mm))
  page-shell(theme, pad(left: gutter)[
    #if data.logo != none {
      image(data.logo, width: 25mm, height: 11mm, fit: "contain")
      v(16pt)
    }
    #heading(level: 1)[
      #show regex("Non-(Disclosure|Circumvention)"): it => box(it)
      #data.kind-label
    ]
    #if data.subtitle != none {
      v(10pt)
      text(size: 12pt, fill: theme.mute)[#data.subtitle]
    }
    #v(23pt)
    #grid(columns: (1fr, 1fr), column-gutter: 12mm,
      party-details(data.our-party, theme), party-details(data.their-party, theme))
    #v(17pt)
    #hairline(theme, weight: 0.4pt)
    #v(12pt)
    #let terms = (("Effective date", data.effective-date-display),
      (if data.kind == "loan" { "Repayment" } else { "Term" }, data.term-short), ("Governing law", data.governing-law))
    #if data.fee-short != none { terms += (("Fees", data.fee-short),) }
    #grid(columns: (29mm, 1fr), column-gutter: 4mm, row-gutter: 5pt,
      ..terms.map(((label, value)) => (quiet(theme, label), text(size: 10.5pt, value))).flatten())
    #v(12pt)
    #hairline(theme, weight: 0.4pt)
    #let clause-content(clause) = [
      #heading(level: 2)[
        #place(top + left, dx: -gutter,
          box(width: 6mm, align(right, text(fill: theme.accent)[#clause.number.])))
        #clause.heading
      ]
      #render-markdown(clause.body)
    ]
    #for (index, clause) in data.clauses.enumerate() {
      if index == data.clauses.len() - 1 {
        // Keep a modest closing clause with execution when the measured
        // content fits comfortably on a page. Long clauses remain flowing.
        block(above: 19pt, layout(size => {
          let closing = [#clause-content(clause)#execution(theme)]
          let height = measure(closing, width: size.width).height
          block(breakable: height > 180mm, closing)
        }))
      } else { clause-content(clause) }
    }
    #if data.clauses.len() == 0 { execution(theme) }
  ])
}
