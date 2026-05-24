// Small layout helpers reused by templates.

#let mm-sp = (xs: 1mm, s: 2mm, m: 4mm, l: 7mm, xl: 12mm)
#let sp    = (xs: 2pt, s: 4pt, m: 8pt, l: 14pt)

#let lbl(theme, text-content) = {
  text(
    size: 7.5pt,
    weight: 500,
    fill: theme.mute,
    tracking: 1pt,
  )[#upper(text-content)]
}

#let hairline(theme, weight: 0.3pt) = {
  line(length: 100%, stroke: weight + theme.hair)
}
