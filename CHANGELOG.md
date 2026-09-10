# Changelog

## 0.3.2

- Set body text to 10.5 pt across all ten templates and reduce titles to 18 pt. Retain 145% baseline spacing, 7 pt extra paragraph separation and consistent 11 pt clause headings.
- Reduce A4 body side margins from 35 mm to 25 mm, top margin from 25 mm to 22 mm and bottom margin from 30 mm to 25 mm. Increase the maximum reading column to 160 mm while preserving A5's narrower layout.
- Update the typography specification and paper-geometry regression checks for the revised defaults.

## 0.3.1

- Retune every document for quieter print typography: 20 pt titles, optically adjusted 10.5–11.5 pt body text, true 145% baseline spacing, 7 pt extra paragraph separation and a centred 140 mm reading column. Record online typography and reading-research sources in `docs/TYPOGRAPHY.md`.
- Correct Archivo font metadata using the upstream regular, medium and semibold faces.
- Keep A4 as default and add A3, A5, US Legal and Executive alongside Letter. Reflow small paper with stacked party details and signatures; retain a comfortable reading measure on large sheets.
- Add `--reference on|off` to rendering and template previews, independently of pagination and the stored document identity.
- Verify all paper sizes across all templates, physical page dimensions, reference visibility and isolated CLI parsing.

## 0.3.0

- Rebuild all ten PDF templates around a shared reading grid, optically sized bundled fonts, hanging clause numbers, deliberate paragraph/list spacing, readable running furniture and consistent signature fields. Keep list introductions with their first item and short list items intact across page breaks.
- Add explicit UK, US, Singapore and global legal profiles; validate state, governing law and venue choices.
- Add folio, counsel and atelier PDF designs, selectable A4/US Letter, embedded font fallbacks and semantic headings in the new designs.
- Add technology, design and startup clause packs. Refresh standard packs for liability, IP, confidentiality, notices, data processing and execution; retain the preceding pack versions in an archive.
- Snapshot selected clause templates and resolve historical contracts using their saved pack version. Reject clean output with unresolved variables.
- Fix the shared database's legacy contract-kind constraint so NCNDA and loan records can be created; preserve populated records, schema objects, sequences and foreign keys. Use distinct number prefixes for every kind.
- Reject invalid terms, contradictory disclosure settings, invalid dates, duration overflow and unsafe template/issuer names. Prevent metadata/clause edits after the first recorded signature and signatures after termination/expiry.
- Preserve wrapped clause/list paragraphs. Compile to a temporary destination and replace the output only on success. Share cached fonts with Typst instead of copying them into every render directory. Keep internal notes out of render data.
- Honour valid configured contract templates, sanitise default output filenames, and refresh automatic titles when duplicating for another party.
- Move to Rust 2024, declare Rust 1.88 minimum, refresh dependencies and track the lockfile. Add storage/lifecycle/profile regression tests, strict linting and a complete PDF content smoke matrix.
- Replace personal example references with fictional data and document legal-profile and signature-record limitations.

Existing installed binaries are not updated by checking out this source. Build and install the desired revision explicitly. Keep your database backup and original signed PDFs when upgrading.
