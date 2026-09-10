//! description: Architectural Swiss typography in Archivo with a firm title and precise hanging numerals.
//! mood: severe, modernist, architectural
//! tags: swiss, brutalist, grid, monochrome, rail, severe, modernist, architectural
//! fonts: Archivo (bundled OFL)
//! paper: white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#191919"), paper: rgb("#FFFFFF"), accent: rgb("#191919"),
  mute: rgb("#555555"), hair: rgb("#C9C9C9"), watermark: rgb("#EEECE8"),
  display-font: "Archivo", body-font: "Archivo", body-size: 11.25pt, title-weight: 600,
)
#modern-contract(theme, character: "basel")
