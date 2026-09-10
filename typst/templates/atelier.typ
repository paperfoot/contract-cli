//! description: Quiet design-studio agreements with expressive type, warm paper and precise spacing.
//! mood: warm, editorial, considered
//! tags: design, creative, studio, branding, architecture, warm
//! fonts: Newsreader 16pt, Libre Franklin (bundled OFL)
//! paper: A4 / warm white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#302B27"), paper: rgb("#FCFAF6"), accent: rgb("#765744"), mute: rgb("#655D55"), hair: rgb("#D8CFC4"), watermark: rgb("#EEE8E0"),
  display-font: "Newsreader 16pt", body-font: "Libre Franklin", label-style: "upper",
  margin: (top: 23mm, bottom: 25mm, left: 27mm, right: 27mm),
)
#modern-contract(theme, character: "atelier")
