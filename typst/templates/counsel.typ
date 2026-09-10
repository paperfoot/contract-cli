//! description: Restrained legal typography with a literary serif and clear numbered clauses.
//! mood: formal, assured, legible
//! tags: legal, formal, lawyer, agreement, serif, classic
//! fonts: Literata, Libre Franklin (bundled OFL)
//! paper: A4 / white
#import "../shared/modern.typ": modern-contract
#let theme = (
  ink: rgb("#252525"), paper: white, accent: rgb("#252525"), mute: rgb("#595959"), hair: rgb("#CECECE"), watermark: rgb("#EEEEEE"),
  display-font: "Literata", body-font: "Literata", label-style: "upper",
  margin: (top: 25mm, bottom: 25mm, left: 28mm, right: 28mm),
)
#modern-contract(theme, character: "counsel")
