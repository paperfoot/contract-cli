# Document typography

Every stock template uses `typst/shared/modern.typ`. Contract kind and clause pack change the content; themes supply font character and colour. The shared layout controls the reading measure, hierarchy, spacing, page furniture and signature fields.

## Evidence and design decisions

The sources below were read on 10 September 2026. They serve different purposes: practitioner guidance supports design choices, reading research describes measured effects, and accessibility guidance addresses a different publication requirement. None establishes one universally correct contract font.

- Matthew Butterick's [point-size guidance](https://typographyforlawyers.com/point-size.html) recommends 10–12 pt for printed body text, adjusted for the font's apparent size. His [heading guidance](https://typographyforlawyers.com/point-size.html) favours small size increments. We use 10.5–11.5 pt body text, headings just 0.5 pt larger, and a 20 pt title.
- His [line-spacing guidance](https://typographyforlawyers.com/line-spacing.html) recommends 120–145% of the type size. We use the spacious end: exactly 145% baseline-to-baseline. His [paragraph guidance](https://typographyforlawyers.com/space-between-paragraphs.html) suggests 4–10 pt of additional separation; we add 7 pt, with no first-line indent.
- His [line-length guidance](https://typographyforlawyers.com/line-length.html) suggests an average of 45–90 characters including spaces. [Margins](https://typographyforlawyers.com/page-margins.html) should follow the reading measure, rather than an arbitrary one-inch default. The reading column is capped at 140 mm, centred, with an 8 mm clause-number gutter outside it. [His contract examples](https://typographyforlawyers.com/contracts.html) also favour more white space and clearer numbering.
- Legge and Bigelow's [2011 vision-science review](https://pmc.ncbi.nlm.nih.gov/articles/PMC3428264/) relates reading to apparent x-height and viewing distance. Nominal point size alone is insufficient. The chosen body fonts have different metrics; their small size adjustments are optical choices, not evidence that one font or size is best for every reader.
- Wallace et al.'s [2022 reading study](https://research.adobe.com/publication/towards-individuated-reading-experiences-different-fonts-increase-reading-speed-for-different-individuals/) found substantial individual variation across fonts. It studied digital reading, not contract comprehension. We retain several font families rather than claiming a universal winner.
- [GOV.UK accessible-document guidance](https://www.gov.uk/guidance/publishing-accessible-documents) recommends at least 12 pt and prefers HTML for public information. These print-oriented defaults are not a claim of accessibility certification or compliance with a prescribed form or court rule. Such requirements and individual reader needs take precedence over these design defaults.

## Type and spacing

| Templates | Body family | Body size | Baseline spacing |
|---|---|---:|---:|
| Folio, Vienna Legal | Libre Franklin | 10.5 pt | 15.225 pt |
| Helvetica Nera, Basel | Archivo | 10.5 pt | 15.225 pt |
| Counsel, Editorial, Marrakech | Literata | 11 pt | 15.95 pt |
| Atelier, Gazette | Newsreader 16pt | 11.5 pt | 16.675 pt |
| Chancery | EB Garamond | 11.5 pt | 16.675 pt |

“Newsreader 16pt” is the font's optical-design family name, not the rendered point size. Gazette uses Fraunces for its title. All body faces have normal width and regular weight; titles use medium or semibold. Fonts are bundled under their OFL licences and embedded in the PDF. Archivo's regular, medium and semibold files come from [upstream revision 2111276](https://github.com/Omnibus-Type/Archivo/tree/211127690e8ff106c36c935f7e5e697114cff103/fonts/ttf); the previous regular file had incorrect internal names.

[Typst's leading](https://typst.app/docs/reference/model/par/#parameters-leading) is the gap between line frames, not the baseline distance. Explicit text edges of `0.8em` and `-0.2em`, plus `0.45em` leading, produce the specified 145% baseline rhythm across font metrics. Paragraph spacing adds 7 pt. Do not substitute a fixed ink gap or assume a word processor's “1.5 lines” means a 150% baseline distance.

Clause headings use 19 pt above and 7 pt below. Titles use natural tracking. Secondary details remain 9.5–10.5 pt, and running headers/page numbers 8.5 pt. Paragraphs are left aligned with normal word spacing. List continuation lines align with item text; introductions stay with the first item. Ordinary short paragraphs and items stay intact, while long custom content can flow. Legal wording is never shortened to hit a page count.

Signature labels remain 9.5 pt with roughly 9 mm blank writing rows. On wider paper the execution block stays intact, with equally tall party-name areas keeping signature lines aligned. A5 stacks party details and signature blocks, allowing page breaks between signatories while keeping each signatory's fields together and the execution introduction with the first signatory.

## Paper and references

A4 is the default for both `render` and `template preview`. Supported [Typst paper definitions](https://typst.app/docs/reference/layout/page/#parameters-paper):

| Flag | Physical size |
|---|---|
| `--paper a4` | 210 × 297 mm |
| `--paper a5` | 148 × 210 mm |
| `--paper a3` | 297 × 420 mm |
| `--paper us-letter` | 8.5 × 11 in |
| `--paper us-legal` | 8.5 × 14 in |
| `--paper us-executive` | 7.25 × 10.5 in |

A4 has 35 mm margins at the body text, 25 mm above and 30 mm below. Clause numbers hang 8 mm into the left margin. Wider pages retain the 140 mm reading measure. A5 uses a 112 mm column with 18 mm body side margins, 18 mm above and 22 mm below. Paper changes reflow content without shrinking the type.

`--reference on` (default) shows the internal document reference in the running header and footer. `--reference off` omits those labels while retaining pagination, the stored identity and any deliberate reference within contractual text. It does not rename the PDF or remove identifiers supplied in the body or filename.

```sh
contract render NDA-acme-2026-0001 --final --paper a4 --reference off
contract template preview counsel --kind nda --paper us-legal --reference on
```

## Verification

Run `python3 scripts/smoke-pdfs.py` after changing layout. It checks all template/kind combinations, all supported paper sizes across every template, actual PDF page dimensions, reference visibility, alternate packs, long custom content and long party names. Text extraction checks content and bounds; it does not establish visual quality.

Render first, middle and final pages and inspect at equivalent physical scale. Measure font names, sizes, baseline distances and line lengths in the PDF itself. Check paragraph rhythm, section spacing, list continuations, header/footer clearance, long names, and writable signatures. Screen zoom or fit-to-width can make the same point size appear very different; physical size must be checked from the PDF page box and text metrics.
