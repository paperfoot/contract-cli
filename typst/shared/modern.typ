#import "contract.typ": data, page-shell, hairline, render-markdown, signature-block, th

// A document-first system: real headings, selectable text, flowing clauses,
// and an execution block that moves intact when space runs out.
#let modern-contract(theme, character: "folio") = {
  set text(font: theme.body-font, size: 10pt, fill: theme.ink, lang: "en", hyphenate: false, number-type: "lining")
  set par(leading: 4.5pt, spacing: 7pt, justify: false)
  set list(indent: 12pt, body-indent: 5pt, spacing: 3pt)
  set enum(indent: 12pt, body-indent: 5pt, spacing: 3pt)
  set heading(numbering: none)
  show heading.where(level: 1): it => block(above: 0pt, below: 0pt, sticky: true)[
    #text(font: theme.display-font, size: if character == "atelier" { 35pt } else if character == "counsel" { 27pt } else { 31pt }, weight: 500, tracking: -0.6pt)[#it.body]
  ]
  show heading.where(level: 2): it => block(above: 12pt, below: 4pt, sticky: true)[
    #text(font: theme.display-font, size: 10.3pt, weight: 600)[#it.body]
  ]
  page-shell(theme, [
    #if data.logo != none { image(data.logo, width: 22mm, height: 10mm, fit: "contain"); v(7mm) }
    #heading(level: 1, data.kind-label)
    #if data.subtitle != none {
      v(4mm)
      text(font: theme.body-font, size: 12pt, fill: theme.mute)[#data.subtitle]
    }
    #v(7mm)
    #text(size: 9pt, fill: theme.mute)[Effective #data.effective-date-display]
    #v(5mm)
    #hairline(theme, weight: 0.6pt)
    #v(4mm)
    #for (i, party) in (data.our-party, data.their-party).enumerate() {
      block(breakable: true, above: 3pt, below: 6pt)[
        #grid(columns: (17mm, 1fr), column-gutter: 4mm,
          text(size: 8.5pt, fill: theme.mute)[#party.role-label],
          [
            #text(weight: 600)[#if party.legal-name != none { party.legal-name } else { party.display-name }]
            #linebreak()
            #text(size: 9pt)[#party.address.join(", ")]
            #if party.company-no != none { linebreak(); text(size: 8.5pt, fill: theme.mute)[Company no. #party.company-no] }
            #if party.email != none { linebreak(); text(size: 8.5pt, fill: theme.mute)[#party.email] }
          ]
        )
      ]
    }
    #v(3mm)
    #hairline(theme)
    #v(4mm)
    #let terms = (("Term", data.term-short), ("Governing law", data.governing-law))
    #if data.fee-short != none { terms += (("Fees", data.fee-short),) }
    #for (label, value) in terms {
      grid(columns: (28mm, 1fr), column-gutter: 3mm,
        text(size: 8.5pt, fill: theme.mute)[#label], text(size: 9pt)[#value])
      v(2pt)
    }
    #v(4mm)
    #hairline(theme)
    #v(2mm)
    #for clause in data.clauses {
      heading(level: 2)[#clause.number.  #clause.heading]
      render-markdown(clause.body)
    }
    #block(breakable: false, above: 10mm)[
      #text(font: theme.display-font, size: 13pt, weight: 500)[Agreement and signatures]
      #v(3mm)
      #text(size: 9pt)[The parties agree to the terms set out above.]
      #v(5mm)
      #signature-block(data.signature, theme)
    ]
  ])
}
