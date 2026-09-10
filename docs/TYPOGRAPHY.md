# Document typography

Every stock template uses `typst/shared/modern.typ`. Contract kind and clause pack change the content, not the layout rules. Themes set bundled fonts, ink and paper colours; page geometry, spacing and execution fields stay shared.

## Reading grid

- A4 and US Letter use 30 mm horizontal margins and 25 mm vertical margins.
- An 8 mm hanging-number gutter leaves a 142 mm reading column on A4. Headings, paragraphs, party details and running furniture share that text axis.
- Paragraphs are flush left without first-line indents. The 5.5 pt line gap and 11.5 pt paragraph gap are ink-to-ink measurements, not baseline distances. Literata receives one extra point in both gaps for its larger apparent letter height.
- Clause headings use the body family in semibold, one point larger than the body, with 19 pt above and 7 pt below. The signature heading uses the same size and weight.
- List continuation lines align with their item text. Introductions stay with the first item; normal items move intact while long custom items can flow across pages. A short terminal word stays with its predecessor.
- Signature fields use readable 9.5 pt labels and approximately 9 mm blank writing rows. The execution block never splits. A closing clause stays with it when their measured combined height fits comfortably on a page. Short paragraphs also move intact.

## Font roles

The sans designs and Literata designs use 11.25 pt body text. Newsreader uses 12 pt and EB Garamond 12.5 pt to compensate for their smaller apparent letter height. These are optical adjustments within one hierarchy. Titles use 27 pt; labels 9.5 pt; running headers and page numbers 8.5 pt. Gazette alone uses a separate title family.

| Templates | Family |
|---|---|
| Folio, Vienna Legal | Libre Franklin |
| Helvetica Nera, Basel | Archivo |
| Counsel, Editorial, Marrakech | Literata |
| Atelier | Newsreader |
| Gazette | Newsreader; Fraunces title |
| Chancery | EB Garamond |

Template names are retained for compatibility. Helvetica Nera now uses bundled Archivo, so it no longer depends on an installed Helvetica face. These design names do not imply deed, witnessing or notarisation functionality.

## Review changes as documents

Run `python3 scripts/smoke-pdfs.py` after changing shared typography. It renders every template and document kind, alternate packs, Letter output, long custom content and long party names. Text extraction verifies content; it cannot establish visual quality.

Render and inspect first, middle and final pages at print scale as well. Check heading and body alignment, paragraph rhythm, list continuations, long party names, header/footer clearance and writable signatures. Do not compress type or spacing to hit an arbitrary page count. Never change legal wording merely to make a page fit.
