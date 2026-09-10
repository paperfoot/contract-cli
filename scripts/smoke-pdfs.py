#!/usr/bin/env python3
"""Render and inspect the complete embedded contract PDF matrix."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unicodedata
import xml.etree.ElementTree as ET


EXPECTED_TEMPLATES = {
    "atelier",
    "basel",
    "chancery",
    "counsel",
    "editorial",
    "folio",
    "gazette",
    "helvetica-nera",
    "marrakech",
    "vienna-legal",
}

KIND_MARKERS = {
    "nda": "potential collaboration on a new software product",
    "ncnda": "neither party is cut out of the introductions made under it",
    "consulting": "design and delivery of a customer-facing dashboard",
    "msa": "specific projects are described in separate statements of work",
    "sow": "acceptance shall be assessed against the deliverables and objective acceptance criteria",
    "service": "commercially reasonable efforts to keep the services available",
    "loan": "borrower may repay the loan early, in whole or in part",
}

KIND_PREFIXES = {
    "nda": "NDA",
    "ncnda": "NCNDA",
    "consulting": "CTR",
    "msa": "MSA",
    "sow": "SOW",
    "service": "SVC",
    "loan": "LOAN",
}

PACK_CASES = (
    (
        "consulting",
        "technology",
        "reasonable secure development practices",
    ),
    (
        "consulting",
        "design",
        "no portfolio permission is implied by payment",
    ),
    (
        "msa",
        "startup",
        "no equity, options, revenue share or success fee is granted",
    ),
)

POINTS_PER_MM = 72.0 / 25.4
PAPER_DIMENSIONS = {
    "a3": (297.0 * POINTS_PER_MM, 420.0 * POINTS_PER_MM),
    "a4": (210.0 * POINTS_PER_MM, 297.0 * POINTS_PER_MM),
    "a5": (148.0 * POINTS_PER_MM, 210.0 * POINTS_PER_MM),
    "us-letter": (612.0, 792.0),
    "us-legal": (612.0, 1008.0),
    "us-executive": (522.0, 756.0),
}
NON_A4_PAPERS = ("a3", "a5", "us-letter", "us-legal", "us-executive")
DIMENSION_TOLERANCE = 0.1
GLYPH_OVERHANG_TOLERANCE = 2.0
# Ink bounds can extend beyond Typst's explicit line frame, particularly for
# Literata headings at a page's top. This only separates body text from running
# furniture; physical page bounds and horizontal grid checks remain strict.
BODY_INK_OVERHANG = 8.0


class SmokeFailure(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render and inspect all embedded contract templates and kinds."
    )
    parser.add_argument(
        "--binary",
        default="target/debug/contract",
        help="contract executable (default: target/debug/contract)",
    )
    return parser.parse_args()


def resolve_binary(value: str) -> Path:
    candidate = Path(value).expanduser()
    if candidate.is_file():
        return candidate.resolve()
    found = shutil.which(value)
    if found:
        return Path(found).resolve()
    raise SmokeFailure(f"contract binary not found: {value}")


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if not path:
        raise SmokeFailure(f"required command not found on PATH: {name}")
    return path


def run_checked(command: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "no output"
        raise SmokeFailure(f"command failed ({result.returncode}): {' '.join(command)}\n{detail}")
    return result


def cli_json(binary: Path, env: dict[str, str], args: list[str]) -> object:
    result = run_checked([str(binary), "--json", *args], env=env)
    try:
        envelope = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise SmokeFailure(f"contract returned invalid JSON for {args}: {error}") from error
    if envelope.get("status") != "success":
        raise SmokeFailure(f"contract returned a non-success envelope for {args}")
    return envelope.get("data")


def normalized(text: str) -> str:
    text = " ".join(unicodedata.normalize("NFKC", text).lower().split())
    # A compound can wrap at its literal hyphen even with hyphenation off.
    # Poppler inserts whitespace there; preserve the hyphen, join the word.
    return re.sub(r"(?<=\w)-\s+(?=\w)", "-", text)


def inspect_pdf(
    pdf: Path,
    pdftotext: str,
    markers: tuple[str, ...],
    *,
    paper: str = "a4",
) -> tuple[str, tuple[str, ...]]:
    if not pdf.is_file():
        raise SmokeFailure(f"preview was not created: {pdf}")
    with pdf.open("rb") as handle:
        if handle.read(5) != b"%PDF-":
            raise SmokeFailure(f"invalid PDF header: {pdf.name}")

    # Body text may span pages. Exclude running furniture by position before
    # joining words, otherwise a page number can falsely break a clause marker.
    result = run_checked([pdftotext, "-bbox", "-enc", "UTF-8", str(pdf), "-"])
    document = ET.fromstring(result.stdout)
    expected_width, expected_height = PAPER_DIMENSIONS[paper]
    expected_width_mm = expected_width / POINTS_PER_MM
    body_width = min(160.0 * POINTS_PER_MM, expected_width - 36.0 * POINTS_PER_MM)
    body_left = (expected_width - body_width) / 2.0
    body_right = body_left + body_width
    body_top = (18.0 if expected_width_mm < 170.0 else 22.0) * POINTS_PER_MM
    body_bottom = (22.0 if expected_width_mm < 170.0 else 25.0) * POINTS_PER_MM
    grid_left = body_left - 8.0 * POINTS_PER_MM - GLYPH_OVERHANG_TOLERANCE
    grid_right = body_right + GLYPH_OVERHANG_TOLERANCE

    pages = document.findall(".//{*}page")
    body_words: list[str] = []
    all_words: list[str] = []
    page_texts: list[str] = []
    for page_number, page in enumerate(pages, start=1):
        width, height = float(page.attrib["width"]), float(page.attrib["height"])
        if (
            abs(width - expected_width) > DIMENSION_TOLERANCE
            or abs(height - expected_height) > DIMENSION_TOLERANCE
        ):
            raise SmokeFailure(
                f"{pdf.name} page {page_number} is {width:.3f} x {height:.3f} pt; "
                f"expected {expected_width:.3f} x {expected_height:.3f} pt for {paper}"
            )
        page_words: list[str] = []
        for word in page.findall(".//{*}word"):
            x0, x1 = float(word.attrib["xMin"]), float(word.attrib["xMax"])
            y0, y1 = float(word.attrib["yMin"]), float(word.attrib["yMax"])
            if x0 < -DIMENSION_TOLERANCE or x1 > width + DIMENSION_TOLERANCE:
                raise SmokeFailure(
                    f"text escapes page {page_number} horizontally in {pdf.name}: {word.text!r}"
                )
            if y0 < -DIMENSION_TOLERANCE or y1 > height + DIMENSION_TOLERANCE:
                raise SmokeFailure(
                    f"text escapes page {page_number} vertically in {pdf.name}: {word.text!r}"
                )
            all_words.append(word.text or "")
            page_words.append(word.text or "")
            if body_top - BODY_INK_OVERHANG <= y0 and y1 <= height - body_bottom + BODY_INK_OVERHANG:
                if x0 < grid_left or x1 > grid_right:
                    raise SmokeFailure(
                        f"text escapes the responsive reading grid on page {page_number} "
                        f"in {pdf.name}: {word.text!r}"
                    )
                body_words.append(word.text or "")
        page_text = normalized(" ".join(page_words))
        if "agreement and signatures" in page_text and (
            "signed by / for" not in page_text or "signature name" not in page_text
        ):
            raise SmokeFailure(
                f"execution introduction is separated from the first signatory "
                f"on page {page_number} in {pdf.name}"
            )
        page_texts.append(page_text)
    text = normalized(" ".join(body_words))
    if "{{" in text or "}}" in text:
        raise SmokeFailure(f"unresolved template variable in {pdf.name}")
    for marker in markers:
        if normalized(marker) not in text:
            raise SmokeFailure(f"missing clause text in {pdf.name}: {marker!r}")
    return normalized(" ".join(all_words)), tuple(page_texts)


def render_preview(
    binary: Path,
    env: dict[str, str],
    pdftotext: str,
    output_dir: Path,
    template: str,
    kind: str,
    *,
    pack: str = "standard",
    paper: str = "a4",
    reference: str = "on",
    extra_marker: str | None = None,
) -> Path:
    filename = f"{template}-{kind}-{pack}-{paper}-reference-{reference}.pdf"
    pdf = output_dir / filename
    cli_json(
        binary,
        env,
        [
            "template",
            "preview",
            template,
            "--kind",
            kind,
            "--pack",
            pack,
            "--paper",
            paper,
            "--reference",
            reference,
            "--out",
            str(pdf),
        ],
    )
    markers = (KIND_MARKERS[kind],) if extra_marker is None else (KIND_MARKERS[kind], extra_marker)
    full_text, _ = inspect_pdf(pdf, pdftotext, markers, paper=paper)
    expected_reference = f"{KIND_PREFIXES[kind]}-acme-2026-0001"
    reference_present = normalized(expected_reference) in full_text
    if reference == "on" and not reference_present:
        raise SmokeFailure(f"preview reference is missing from {pdf.name}")
    if reference == "off" and reference_present:
        raise SmokeFailure(f"preview reference is visible despite --reference off in {pdf.name}")
    return pdf


def isolated_environment(root: Path) -> dict[str, str]:
    home = root / "home"
    config = root / "config"
    data = root / "data"
    cache = root / "cache"
    for directory in (home, config, data, cache):
        directory.mkdir()
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
            "XDG_CONFIG_HOME": str(config),
            "XDG_DATA_HOME": str(data),
            "XDG_CACHE_HOME": str(cache),
        }
    )
    return env


def check_custom_content(binary: Path, env: dict[str, str], pdftotext: str, root: Path) -> int:
    cli_json(binary, env, ["issuer", "add", "example", "--name", "Example Studio", "--jurisdiction", "uk", "--address", "1 Example Street"])
    cli_json(binary, env, ["clients", "add", "client", "--name", "Example Client", "--address", "2 Example Street"])
    record = cli_json(binary, env, ["new", "--kind", "nda", "--as", "example", "--client", "client", "--legal-profile", "us", "--us-state", "Delaware", "--purpose", "A synthetic render regression", "--notes", "PRIVATE_NOTE_SENTINEL"])
    number = record["number"]
    markers = ["Paragraph sentinel", "Bullet continuation sentinel", "Number continuation sentinel", "Final paragraph sentinel"]
    body = "Paragraph sentinel. Literal #panic(42) must remain text.\n\n- Bullet start\n  Bullet continuation sentinel\n\n1. Number start\n   Number continuation sentinel\n\n"
    for index in range(40):
        marker = f"Long item {index:03d} complete"
        markers.append(marker)
        body += f"- {marker}. " + "Additional agreed scope details remain readable across page breaks. " * 3 + "\n"
    body += "\nFinal paragraph sentinel."
    source = root / "clause.md"
    source.write_text(body)
    cli_json(binary, env, ["contracts", "clauses", "add", number, "regression", "--heading", "Additional terms", "--from-file", str(source)])
    required_markers = tuple(
        markers + ["report or investigate a suspected violation of law"]
    )
    reference_on_pdf = root / "custom-reference-on.pdf"
    reference_off_pdf = root / "custom-reference-off.pdf"
    cli_json(
        binary,
        env,
        [
            "render",
            number,
            "--template",
            "folio",
            "--final",
            "--reference",
            "on",
            "--out",
            str(reference_on_pdf),
        ],
    )
    on_text, on_pages = inspect_pdf(
        reference_on_pdf, pdftotext, required_markers, paper="a4"
    )
    cli_json(
        binary,
        env,
        [
            "render",
            number,
            "--template",
            "folio",
            "--final",
            "--reference",
            "off",
            "--out",
            str(reference_off_pdf),
        ],
    )
    off_text, off_pages = inspect_pdf(
        reference_off_pdf, pdftotext, required_markers, paper="a4"
    )
    normalized_number = normalized(number)
    if normalized_number not in on_text:
        raise SmokeFailure("--reference on omitted the real contract number")
    if normalized_number in off_text:
        raise SmokeFailure("--reference off left the real contract number in the PDF")
    if len(on_pages) != len(off_pages):
        raise SmokeFailure("reference visibility changed the real contract page count")

    on_without_reference = normalized(on_text.replace(normalized_number, " "))
    if on_without_reference != off_text:
        raise SmokeFailure("reference visibility changed real contract clause text")
    page_count = len(on_pages)
    for page_number, (on_page, off_page) in enumerate(zip(on_pages, off_pages), start=1):
        pagination = re.compile(rf"\b{page_number}\s*/\s*{page_count}\b")
        if not pagination.search(on_page) or not pagination.search(off_page):
            raise SmokeFailure(
                f"reference visibility removed pagination from real contract page {page_number}"
            )

    shown = cli_json(binary, env, ["show", number])
    if not isinstance(shown, dict) or shown.get("number") != number:
        raise SmokeFailure("reference visibility changed the internally stored contract number")

    contents = run_checked([pdftotext, str(reference_off_pdf), "-"]).stdout
    if "PRIVATE_NOTE_SENTINEL" in contents:
        raise SmokeFailure("private notes entered the PDF")
    original = reference_on_pdf.read_bytes()

    # Force a real compiler failure while preserving the existing destination.
    fake_bin = root / "fake-bin"
    fake_bin.mkdir()
    fake_typst = fake_bin / "typst"
    fake_typst.write_text("#!/bin/sh\nprintf 'synthetic compiler failure' >&2\nexit 1\n")
    fake_typst.chmod(0o755)
    failed_env = dict(env, PATH=str(fake_bin) + os.pathsep + env.get("PATH", ""))
    failed = subprocess.run([str(binary), "--json", "render", number, "--template", "folio", "--final", "--reference", "on", "--out", str(reference_on_pdf)], env=failed_env, capture_output=True, text=True)
    if failed.returncode == 0 or reference_on_pdf.read_bytes() != original or failed.stdout:
        raise SmokeFailure("compiler failure replaced output or contaminated stdout")
    json.loads(failed.stderr)

    cli_json(binary, env, ["contracts", "clauses", "edit", number, "regression", "--body", "Required {{missing_term}}"])
    failed = subprocess.run([str(binary), "--json", "render", number, "--template", "folio", "--final", "--reference", "on", "--out", str(reference_on_pdf)], env=env, capture_output=True, text=True)
    if failed.returncode != 3 or reference_on_pdf.read_bytes() != original:
        raise SmokeFailure("unresolved term did not block a clean render")
    return 2


def check_long_parties(
    binary: Path,
    env: dict[str, str],
    pdftotext: str,
    output_dir: Path,
) -> int:
    issuer_legal = (
        "Acme Example International Software Research, Product Design, Systems "
        "Engineering and Responsible Innovation Holdings Limited"
    )
    client_legal = (
        "Meridian Example Global Technology Advisory, Digital Infrastructure, "
        "Commercial Strategy and Sustainable Ventures Limited"
    )
    cli_json(
        binary,
        env,
        [
            "issuer",
            "add",
            "acme-long",
            "--name",
            "Acme Example",
            "--legal-name",
            issuer_legal,
            "--jurisdiction",
            "uk",
            "--address",
            "100 Example Way\nExample District\nLondon EX1 1AA\nUnited Kingdom",
            "--email",
            "contracts-and-legal-notices-for-international-projects@acme-long.example",
        ],
    )
    cli_json(
        binary,
        env,
        [
            "clients",
            "add",
            "meridian-long",
            "--name",
            "Meridian Example",
            "--legal-name",
            client_legal,
            "--jurisdiction",
            "England and Wales",
            "--address",
            "200 Sample Avenue\nSample Quarter\nManchester EX2 2BB\nUnited Kingdom",
            "--email",
            "legal-and-procurement-correspondence@meridian-long.example",
        ],
    )
    record = cli_json(
        binary,
        env,
        [
            "new",
            "--kind",
            "consulting",
            "--as",
            "acme-long",
            "--client",
            "meridian-long",
            "--purpose",
            "Evaluate a software partnership",
            "--fee",
            "fixed:8400:GBP",
            "--legal-profile",
            "uk",
            "--pack",
            "standard",
        ],
    )
    number = record["number"]
    markers = (
        issuer_legal,
        client_legal,
        "Agreement and signatures",
        "The parties agree to the terms set out above.",
        "Signed by / for",
    )
    rendered = 0
    for template in ("folio", "counsel"):
        pdf = output_dir / f"{template}-consulting-long-parties.pdf"
        cli_json(
            binary,
            env,
            [
                "render",
                number,
                "--template",
                template,
                "--final",
                "--out",
                str(pdf),
            ],
        )
        inspect_pdf(pdf, pdftotext, markers)
        rendered += 1
    return rendered


def main() -> int:
    args = parse_args()
    binary = resolve_binary(args.binary)
    require_tool("typst")
    pdftotext = require_tool("pdftotext")

    with tempfile.TemporaryDirectory(prefix="contract-cli-pdf-smoke-") as temporary:
        root = Path(temporary)
        output_dir = root / "pdfs"
        output_dir.mkdir()
        env = isolated_environment(root)

        listing = cli_json(binary, env, ["template", "list"])
        if not isinstance(listing, list):
            raise SmokeFailure("template list data is not an array")
        templates = [item.get("name") for item in listing if isinstance(item, dict)]
        if len(templates) != 10 or set(templates) != EXPECTED_TEMPLATES:
            raise SmokeFailure(
                f"expected 10 embedded templates {sorted(EXPECTED_TEMPLATES)}, got {sorted(templates)}"
            )

        rendered = 0
        for template in sorted(templates):
            for kind in KIND_MARKERS:
                render_preview(binary, env, pdftotext, output_dir, template, kind)
                rendered += 1

        for template in sorted(templates):
            for paper in NON_A4_PAPERS:
                render_preview(
                    binary,
                    env,
                    pdftotext,
                    output_dir,
                    template,
                    "consulting",
                    paper=paper,
                    reference="off",
                )
                rendered += 1

        # These closing clauses previously stranded the execution introduction
        # on the preceding A5 page, away from both signatories.
        for template, kind in (("counsel", "nda"), ("folio", "ncnda")):
            render_preview(
                binary, env, pdftotext, output_dir, template, kind,
                paper="a5", reference="off",
            )
            rendered += 1

        for kind, pack, marker in PACK_CASES:
            render_preview(
                binary,
                env,
                pdftotext,
                output_dir,
                "folio",
                kind,
                pack=pack,
                extra_marker=marker,
            )
            rendered += 1

        rendered += check_custom_content(binary, env, pdftotext, root)
        rendered += check_long_parties(binary, env, pdftotext, output_dir)

    print(
        f"OK: {rendered} PDFs; 10 templates x 7 kinds, 3 pack previews, "
        f"10 templates x 5 non-A4 sizes with references off, "
        f"2 A5 execution regressions, "
        f"2 reference-state long custom documents, 2 long-party final documents; "
        f"dimensions, pagination, execution, failure and notes checks passed"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SmokeFailure as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
