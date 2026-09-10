//! description: Sober Swiss typography in bundled Archivo, black on white. The house default.
//! mood: corporate, sober, modern
//! tags: swiss, minimal, monochrome, corporate, modern, sans-serif, clean, black-white, helvetica, plain
//! fonts: Archivo (bundled OFL)
//! paper: white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#202020"), paper: rgb("#FFFFFF"), accent: rgb("#202020"),
  mute: rgb("#5A5A5A"), hair: rgb("#CCCCCC"), watermark: rgb("#EEECE8"),
  display-font: "Archivo", body-font: "Archivo", body-size: 11.25pt, title-weight: 500,
)
#modern-contract(theme, character: "helvetica-nera")
