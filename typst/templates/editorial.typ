//! description: Literary agreements in Literata on soft ivory with a consistent left-aligned reading grid.
//! mood: formal, classic, literary
//! tags: serif, classic, formal, traditional, elegant, literary, editorial
//! fonts: Literata (bundled OFL)
//! paper: cream
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#302D28"), paper: rgb("#FCFAF5"), accent: rgb("#4E473D"),
  mute: rgb("#645F55"), hair: rgb("#D7D0C4"), watermark: rgb("#EEECE8"),
  display-font: "Literata", body-font: "Literata", body-size: 11pt, title-weight: 500,
)
#modern-contract(theme, character: "editorial")
