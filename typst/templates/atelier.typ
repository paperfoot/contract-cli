//! description: Quiet design-studio agreements in Newsreader on warm white paper.
//! mood: warm, editorial, considered
//! tags: design, creative, studio, branding, architecture, warm
//! fonts: Newsreader 16pt (bundled OFL)
//! paper: A4 / warm white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#302B27"), paper: rgb("#FCFAF6"), accent: rgb("#765744"),
  mute: rgb("#655D55"), hair: rgb("#D8CFC4"), watermark: rgb("#EEECE8"),
  display-font: "Newsreader 16pt", body-font: "Newsreader 16pt", body-size: 12pt, title-weight: 500,
)
#modern-contract(theme, character: "atelier")
