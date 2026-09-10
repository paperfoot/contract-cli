//! description: Precise technology and startup agreements with a clear sans serif and generous reading space.
//! mood: precise, calm, contemporary
//! tags: technology, software, startup, modern, clean, white
//! fonts: Libre Franklin (bundled OFL)
//! paper: A4 / white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#202722"), paper: rgb("#FFFFFF"), accent: rgb("#33483E"),
  mute: rgb("#58615B"), hair: rgb("#CCD3CE"), watermark: rgb("#EEECE8"),
  display-font: "Libre Franklin", body-font: "Libre Franklin", body-size: 10.5pt, title-weight: 500,
)
#modern-contract(theme, character: "folio")
