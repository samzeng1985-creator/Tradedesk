#!/usr/bin/env python3
"""Generate and verify TradeDesk release fixtures without adding app dependencies."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import pdfplumber
from PIL import Image, ImageDraw
from pypdf import PdfReader


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run TradeDesk 0.26 release QA")
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output-root", type=Path, default=Path("output/release-qa/0.26.0"))
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("tests/fixtures/pdf-golden-manifest.json"),
    )
    parser.add_argument("--skip-tests", action="store_true")
    parser.add_argument("--skip-performance", action="store_true")
    parser.add_argument("--pdf-dir", type=Path, help="Verify an existing PDF directory")
    return parser.parse_args()


def resolve_command(name: str) -> str:
    command = shutil.which(name)
    if not command:
        raise RuntimeError(f"Required release tool is missing: {name}")
    command_path = Path(command)
    if os.name == "nt" and command_path.suffix.lower() in {".cmd", ".bat"}:
        bundled_executable = (
            command_path.parents[2] / "native" / "poppler" / "Library" / "bin" / f"{name}.exe"
        )
        if bundled_executable.exists():
            return str(bundled_executable)
    return command


def command_line(executable: str, *arguments: str) -> list[str]:
    if os.name == "nt" and Path(executable).suffix.lower() in {".cmd", ".bat"}:
        return [os.environ.get("COMSPEC", "cmd.exe"), "/d", "/c", executable, *arguments]
    return [executable, *arguments]


def run(command: list[str], repo: Path, environment: dict[str, str]) -> float:
    started = time.perf_counter()
    subprocess.run(command, cwd=repo, env=environment, check=True)
    return round(time.perf_counter() - started, 3)


def render_pdf(pdftoppm: str, pdf_path: Path, render_root: Path) -> list[Path]:
    document_root = render_root / pdf_path.stem
    document_root.mkdir(parents=True, exist_ok=True)
    prefix = document_root / "page"
    subprocess.run(
        command_line(pdftoppm, "-png", "-r", "120", str(pdf_path), str(prefix)),
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return sorted(document_root.glob("page-*.png"))


def inspect_pdf(
    pdf_path: Path,
    render_root: Path,
    pdftoppm: str,
    defaults: dict[str, int],
) -> tuple[dict[str, object], list[str]]:
    failures: list[str] = []
    reader = PdfReader(str(pdf_path))
    page_count = len(reader.pages)
    if not defaults["minPages"] <= page_count <= defaults["maxPages"]:
        failures.append(
            f"{pdf_path.name}: page count {page_count} is outside "
            f"{defaults['minPages']}..{defaults['maxPages']}"
        )

    text_characters = 0
    word_count = 0
    out_of_bounds_words = 0
    page_sizes: list[dict[str, float]] = []
    with pdfplumber.open(pdf_path) as document:
        for page_number, page in enumerate(document.pages, start=1):
            width = float(page.width)
            height = float(page.height)
            page_sizes.append({"width": round(width, 2), "height": round(height, 2)})
            if width < 300 or height < 300 or width > 1000 or height > 1000:
                failures.append(
                    f"{pdf_path.name} page {page_number}: unexpected page size {width}x{height}"
                )
            text = page.extract_text() or ""
            text_characters += len(text.strip())
            words = page.extract_words()
            word_count += len(words)
            for word in words:
                if (
                    float(word["x0"]) < -1
                    or float(word["x1"]) > width + 1
                    or float(word["top"]) < -1
                    or float(word["bottom"]) > height + 1
                ):
                    out_of_bounds_words += 1
                    failures.append(
                        f"{pdf_path.name} page {page_number}: text outside page: {word['text']!r}"
                    )

    if text_characters < defaults["minTextCharacters"]:
        failures.append(
            f"{pdf_path.name}: only {text_characters} extractable text characters"
        )

    rendered_pages = render_pdf(pdftoppm, pdf_path, render_root)
    if len(rendered_pages) != page_count:
        failures.append(
            f"{pdf_path.name}: rendered {len(rendered_pages)} pages, expected {page_count}"
        )
    rendered_metrics: list[dict[str, object]] = []
    for page_number, image_path in enumerate(rendered_pages, start=1):
        with Image.open(image_path) as image:
            grayscale = image.convert("L")
            histogram = grayscale.histogram()
            ink_pixels = sum(histogram[:250])
            total_pixels = grayscale.width * grayscale.height
            ink_ratio = ink_pixels / total_pixels if total_pixels else 0
            rendered_metrics.append(
                {
                    "page": page_number,
                    "width": grayscale.width,
                    "height": grayscale.height,
                    "inkRatio": round(ink_ratio, 6),
                }
            )
            if ink_ratio < 0.0005:
                failures.append(f"{pdf_path.name} page {page_number}: rendered page is blank")

    return (
        {
            "file": pdf_path.name,
            "sha256": hashlib.sha256(pdf_path.read_bytes()).hexdigest(),
            "bytes": pdf_path.stat().st_size,
            "pages": page_count,
            "pageSizes": page_sizes,
            "textCharacters": text_characters,
            "words": word_count,
            "outOfBoundsWords": out_of_bounds_words,
            "renderedPages": rendered_metrics,
        },
        failures,
    )


def write_contact_sheet(render_root: Path, output_path: Path, file_names: list[str]) -> None:
    thumbnails: list[tuple[str, Image.Image]] = []
    for file_name in file_names:
        first_page = render_root / Path(file_name).stem / "page-1.png"
        if not first_page.exists():
            continue
        with Image.open(first_page) as image:
            thumbnail = image.convert("RGB")
            thumbnail.thumbnail((300, 420))
            thumbnails.append((Path(file_name).stem, thumbnail.copy()))
    if not thumbnails:
        return
    columns = 4
    cell_width, cell_height = 330, 465
    rows = (len(thumbnails) + columns - 1) // columns
    sheet = Image.new("RGB", (columns * cell_width, rows * cell_height), "white")
    draw = ImageDraw.Draw(sheet)
    for index, (label, thumbnail) in enumerate(thumbnails):
        x = (index % columns) * cell_width + 15
        y = (index // columns) * cell_height + 30
        sheet.paste(thumbnail, (x, y))
        draw.text((x, 8 + (index // columns) * cell_height), label, fill="black")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(output_path)


def main() -> int:
    args = parse_args()
    repo = args.repo.resolve()
    output_root = args.output_root
    if not output_root.is_absolute():
        output_root = (repo / output_root).resolve()
    manifest_path = args.manifest
    if not manifest_path.is_absolute():
        manifest_path = (repo / manifest_path).resolve()
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_root = output_root / run_id
    pdf_root = args.pdf_dir.resolve() if args.pdf_dir else run_root / "pdfs"
    render_root = run_root / "renders"
    report_path = run_root / "release-qa-report.json"
    pdf_root.mkdir(parents=True, exist_ok=True)
    render_root.mkdir(parents=True, exist_ok=True)

    environment = os.environ.copy()
    environment["TRADEDESK_PDF_QA_DIR"] = str(pdf_root)
    environment["TRADEDESK_PERF_REPORT"] = str(run_root / "performance.json")
    durations: dict[str, float] = {}
    if not args.skip_tests:
        durations["frontendTests"] = run(
            command_line(resolve_command("pnpm"), "test"), repo, environment
        )
        durations["frontendBuild"] = run(
            command_line(resolve_command("pnpm"), "build"), repo, environment
        )
        durations["rustTests"] = run(
            command_line(
                resolve_command("cargo"),
                "test",
                "--manifest-path",
                "src-tauri/Cargo.toml",
            ),
            repo,
            environment,
        )
    if not args.skip_performance:
        durations["performanceTest"] = run(
            command_line(
                resolve_command("cargo"),
                "test",
                "--manifest-path",
                "src-tauri/Cargo.toml",
                "release_large_dataset_performance",
                "--",
                "--ignored",
                "--nocapture",
            ),
            repo,
            environment,
        )

    pdftoppm = resolve_command("pdftoppm")
    expected_files: list[str] = manifest["files"]
    actual_files = sorted(path.name for path in pdf_root.glob("*.pdf"))
    failures = [
        f"Missing expected PDF: {file_name}"
        for file_name in expected_files
        if file_name not in actual_files
    ]
    failures.extend(
        f"Unexpected PDF: {file_name}"
        for file_name in actual_files
        if file_name not in expected_files
    )
    file_reports: list[dict[str, object]] = []
    for file_name in expected_files:
        pdf_path = pdf_root / file_name
        if not pdf_path.exists():
            continue
        report, file_failures = inspect_pdf(
            pdf_path, render_root, pdftoppm, manifest["defaults"]
        )
        file_reports.append(report)
        failures.extend(file_failures)

    contact_sheet = run_root / "pdf-contact-sheet.png"
    write_contact_sheet(render_root, contact_sheet, expected_files)
    report = {
        "release": manifest["release"],
        "createdAt": datetime.now(timezone.utc).isoformat(),
        "platform": platform.platform(),
        "python": sys.version.split()[0],
        "durationsSeconds": durations,
        "pdfDirectory": str(pdf_root),
        "renderDirectory": str(render_root),
        "contactSheet": str(contact_sheet),
        "expectedFiles": len(expected_files),
        "verifiedFiles": len(file_reports),
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "files": file_reports,
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({"status": report["status"], "report": str(report_path)}, indent=2))
    if failures:
        for failure in failures:
            print(f"ERROR: {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
