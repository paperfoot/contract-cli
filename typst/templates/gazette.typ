//! description: An expressive Fraunces masthead with calm Newsreader text and carefully spaced clauses.
//! mood: literary, statement, editorial
//! tags: magazine, masthead, editorial, serif, white, ragged, statement, broadsheet, newspaper
//! fonts: Newsreader 16pt, Fraunces 72pt (bundled OFL)
//! paper: white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#252525"), paper: rgb("#FFFFFF"), accent: rgb("#252525"),
  mute: rgb("#595959"), hair: rgb("#CCCCCC"), watermark: rgb("#EEECE8"),
  display-font: "Fraunces 72pt", body-font: "Newsreader 16pt", body-size: 12pt, title-weight: 600,
)
#modern-contract(theme, character: "gazette")
