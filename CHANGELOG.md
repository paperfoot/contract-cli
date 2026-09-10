# Changelog

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
