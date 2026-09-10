//! description: Restrained legal typography in Literata with hanging clause numbers and a calm reading rhythm.
//! mood: formal, assured, legible
//! tags: legal, formal, lawyer, agreement, serif, classic
//! fonts: Literata (bundled OFL)
//! paper: A4 / white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#252525"), paper: rgb("#FFFFFF"), accent: rgb("#252525"),
  mute: rgb("#595959"), hair: rgb("#CECECE"), watermark: rgb("#EEECE8"),
  display-font: "Literata", body-font: "Literata", body-size: 11.25pt, title-weight: 500,
)
#modern-contract(theme, character: "counsel")
