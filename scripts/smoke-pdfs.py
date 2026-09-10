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
    return " ".join(unicodedata.normalize("NFKC", text).lower().split())


def inspect_pdf(pdf: Path, pdftotext: str, markers: tuple[str, ...]) -> None:
    if not pdf.is_file():
        raise SmokeFailure(f"preview was not created: {pdf}")
    with pdf.open("rb") as handle:
        if handle.read(5) != b"%PDF-":
            raise SmokeFailure(f"invalid PDF header: {pdf.name}")

    # Body text may span pages. Exclude running furniture by position before
    # joining words, otherwise a page number can falsely break a clause marker.
    result = run_checked([pdftotext, "-bbox", "-enc", "UTF-8", str(pdf), "-"])
    document = ET.fromstring(result.stdout)
    body_words = []
    for page in document.findall(".//{*}page"):
        width, height = float(page.attrib["width"]), float(page.attrib["height"])
        for word in page.findall(".//{*}word"):
            x0, x1 = float(word.attrib["xMin"]), float(word.attrib["xMax"])
            y0, y1 = float(word.attrib["yMin"]), float(word.attrib["yMax"])
            if 65 <= y0 and y1 <= height - 65:
                if x0 < 65 or x1 > width - 80:
                    raise SmokeFailure(f"text escapes the reading grid in {pdf.name}: {word.text!r}")
                body_words.append(word.text or "")
    text = normalized(" ".join(body_words))
    if "{{" in text or "}}" in text:
        raise SmokeFailure(f"unresolved template variable in {pdf.name}")
    for marker in markers:
        if normalized(marker) not in text:
            raise SmokeFailure(f"missing clause text in {pdf.name}: {marker!r}")


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
    extra_marker: str | None = None,
) -> Path:
    filename = f"{template}-{kind}-{pack}-{paper}.pdf"
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
            "--out",
            str(pdf),
        ],
    )
    markers = (KIND_MARKERS[kind],) if extra_marker is None else (KIND_MARKERS[kind], extra_marker)
    inspect_pdf(pdf, pdftotext, markers)
    return pdf


def check_us_letter(pdf: Path, pdfinfo: str) -> None:
    result = run_checked([pdfinfo, str(pdf)])
    match = re.search(r"^Page size:\s+([0-9.]+) x ([0-9.]+) pts", result.stdout, re.MULTILINE)
    if not match:
        raise SmokeFailure(f"pdfinfo did not report page dimensions for {pdf.name}")
    width, height = (float(value) for value in match.groups())
    if abs(width - 612.0) > 1.0 or abs(height - 792.0) > 1.0:
        raise SmokeFailure(
            f"{pdf.name} is {width:g} x {height:g} pt; expected US Letter 612 x 792 pt"
        )


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


def check_custom_content(binary: Path, env: dict[str, str], pdftotext: str, root: Path) -> None:
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
    pdf = root / "custom.pdf"
    cli_json(binary, env, ["render", number, "--template", "folio", "--final", "--out", str(pdf)])
    inspect_pdf(pdf, pdftotext, tuple(markers + ["report or investigate a suspected violation of law"]))
    contents = run_checked([pdftotext, str(pdf), "-"]).stdout
    if "PRIVATE_NOTE_SENTINEL" in contents:
        raise SmokeFailure("private notes entered the PDF")
    original = pdf.read_bytes()

    # Force a real compiler failure while preserving the existing destination.
    fake_bin = root / "fake-bin"
    fake_bin.mkdir()
    fake_typst = fake_bin / "typst"
    fake_typst.write_text("#!/bin/sh\nprintf 'synthetic compiler failure' >&2\nexit 1\n")
    fake_typst.chmod(0o755)
    failed_env = dict(env, PATH=str(fake_bin) + os.pathsep + env.get("PATH", ""))
    failed = subprocess.run([str(binary), "--json", "render", number, "--template", "folio", "--final", "--out", str(pdf)], env=failed_env, capture_output=True, text=True)
    if failed.returncode == 0 or pdf.read_bytes() != original or failed.stdout:
        raise SmokeFailure("compiler failure replaced output or contaminated stdout")
    json.loads(failed.stderr)

    cli_json(binary, env, ["contracts", "clauses", "edit", number, "regression", "--body", "Required {{missing_term}}"])
    failed = subprocess.run([str(binary), "--json", "render", number, "--template", "folio", "--final", "--out", str(pdf)], env=env, capture_output=True, text=True)
    if failed.returncode != 3 or pdf.read_bytes() != original:
        raise SmokeFailure("unresolved term did not block a clean render")


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
    pdfinfo = shutil.which("pdfinfo")

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

        letter_pdf = render_preview(
            binary,
            env,
            pdftotext,
            output_dir,
            "folio",
            "consulting",
            paper="us-letter",
        )
        rendered += 1
        if pdfinfo:
            check_us_letter(letter_pdf, pdfinfo)
        check_custom_content(binary, env, pdftotext, root)
        rendered += 1
        rendered += check_long_parties(binary, env, pdftotext, output_dir)

    dimension_status = "checked" if pdfinfo else "skipped (pdfinfo unavailable)"
    print(
        f"OK: {rendered} PDFs; 10 templates x 7 kinds, 3 pack previews, "
        f"1 US Letter ({dimension_status}), 1 long custom document, "
        f"2 long-party final documents; failure/notes checks passed"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SmokeFailure as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
