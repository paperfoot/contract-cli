//! description: Precise technology and startup agreements with generous type and clean white pages.
//! mood: precise, calm, contemporary
//! tags: technology, software, startup, modern, clean, white
//! fonts: Archivo, Libre Franklin (bundled OFL)
//! paper: A4 / white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#202722"), paper: white, accent: rgb("#33483E"), mute: rgb("#58615B"), hair: rgb("#CCD3CE"), watermark: rgb("#EDF0ED"),
  display-font: "Archivo", body-font: "Libre Franklin", label-style: "upper",
  margin: (top: 22mm, bottom: 24mm, left: 26mm, right: 26mm),
)
#modern-contract(theme)
