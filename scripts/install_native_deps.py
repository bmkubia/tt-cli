#!/usr/bin/env python3
"""Download tt CLI build artifacts and install them into a vendor/ directory."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


DEFAULT_REPO = "bmkubia/tt-cli"
REPO = os.environ.get("TT_RELEASE_REPO", DEFAULT_REPO)
BINARY_TARGETS = {
    "x86_64-unknown-linux-musl": "tt",
    "aarch64-unknown-linux-musl": "tt",
    "x86_64-apple-darwin": "tt",
    "aarch64-apple-darwin": "tt",
    "x86_64-pc-windows-msvc": "tt.exe",
}
ARTIFACT_BINARY_TEMPLATE = "tt-{target}{suffix}"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workflow-url",
        required=True,
        help="GitHub Actions workflow URL that produced the desired artifacts.",
    )
    parser.add_argument(
        "--component",
        dest="components",
        action="append",
        choices=("tt",),
        help="Component(s) to install. Defaults to 'tt'.",
    )
    parser.add_argument(
        "vendor_root",
        type=Path,
        help="Directory that will receive the vendor/<target>/tt binaries.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    vendor_root = args.vendor_root.resolve()
    vendor_root.mkdir(parents=True, exist_ok=True)

    components = args.components or ["tt"]
    if "tt" not in components:
        print("No components requested; nothing to do.")
        return 0

    run_id = extract_run_id(args.workflow_url)

    with tempfile.TemporaryDirectory(prefix="tt-cli-artifacts-") as tmp:
        archive_root = Path(tmp)
        download_artifacts(run_id, archive_root)
        install_tt_binaries(archive_root, vendor_root)

    print(f"Installed tt binaries into {vendor_root}")
    return 0


def extract_run_id(url: str) -> str:
    parts = url.rstrip("/").split("/")
    if not parts or not parts[-1].isdigit():
        raise RuntimeError(f"Unable to parse run id from {url}")
    return parts[-1]


def download_artifacts(run_id: str, dest: Path) -> None:
    print(f"Downloading artifacts for run {run_id}...")
    cmd = [
        "gh",
        "run",
        "download",
        run_id,
        "--repo",
        REPO,
        "--dir",
        str(dest),
    ]
    subprocess.run(cmd, check=True)


def install_tt_binaries(archive_root: Path, vendor_root: Path) -> None:
    for target, binary_name in BINARY_TARGETS.items():
        artifact_dir = archive_root / target
        if not artifact_dir.exists():
            raise RuntimeError(f"Artifact directory missing for {target}: {artifact_dir}")

        suffix = ".exe" if binary_name.endswith(".exe") else ""
        artifact_binary_name = ARTIFACT_BINARY_TEMPLATE.format(target=target, suffix=suffix)
        src_binary = artifact_dir / artifact_binary_name
        if not src_binary.exists():
            raise RuntimeError(f"Binary {src_binary} not found for {target}")

        dest_dir = vendor_root / target / "tt"
        dest_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src_binary, dest_dir / binary_name)


if __name__ == "__main__":
    raise SystemExit(main())
