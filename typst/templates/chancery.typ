//! description: Classic agreements in EB Garamond with generous leading and a clear execution block.
//! mood: formal, engraved, ceremonial
//! tags: engraved, formal, garamond, small-caps, classic, chancery
//! fonts: EB Garamond (bundled OFL)
//! paper: ivory
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#2D2B26"), paper: rgb("#FDFBF6"), accent: rgb("#494338"),
  mute: rgb("#655F53"), hair: rgb("#D7D0C3"), watermark: rgb("#EEECE8"),
  display-font: "EB Garamond", body-font: "EB Garamond", body-size: 10.5pt, title-weight: 500,
)
#modern-contract(theme, character: "chancery")
