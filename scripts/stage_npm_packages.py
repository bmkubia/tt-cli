#!/usr/bin/env python3
"""Stage one or more npm packages for tt-cli releases."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BUILD_SCRIPT = REPO_ROOT / "scripts" / "build_npm_package.py"
INSTALL_NATIVE_DEPS = REPO_ROOT / "scripts" / "install_native_deps.py"
PACKAGE_NATIVE_COMPONENTS = {
    "tt-cli": ["tt"],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--release-version",
        required=True,
        help="Version to stage (e.g. 0.1.0 or 0.1.0-alpha.1).",
    )
    parser.add_argument(
        "--package",
        dest="packages",
        action="append",
        choices=tuple(PACKAGE_NATIVE_COMPONENTS.keys()),
        default=None,
        help="Package name to stage. Defaults to tt-cli.",
    )
    parser.add_argument(
        "--workflow-url",
        required=True,
        help="GitHub Actions workflow URL that produced the artifacts.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Directory where npm tarballs should be written (default: dist/npm).",
    )
    parser.add_argument(
        "--keep-staging-dirs",
        action="store_true",
        help="Retain temporary staging directories instead of deleting them.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    packages = list(args.packages or ["tt-cli"])
    native_components = collect_native_components(packages)

    output_dir = args.output_dir or (REPO_ROOT / "dist" / "npm")
    output_dir.mkdir(parents=True, exist_ok=True)

    vendor_temp_root: Path | None = None
    vendor_src: Path | None = None

    try:
        if native_components:
            vendor_temp_root = Path(tempfile.mkdtemp(prefix="tt-cli-vendor-"))
            install_native_components(args.workflow_url, native_components, vendor_temp_root)
            vendor_src = vendor_temp_root

        for package in packages:
            staging_dir = Path(tempfile.mkdtemp(prefix=f"npm-stage-{package}-"))
            pack_output = output_dir / f"{package}-{args.release_version}.tgz"

            if vendor_src is None:
                raise RuntimeError("Vendor directory was not prepared.")

            cmd = [
                str(BUILD_SCRIPT),
                "--package",
                package,
                "--release-version",
                args.release_version,
                "--staging-dir",
                str(staging_dir),
                "--pack-output",
                str(pack_output),
                "--vendor-src",
                str(vendor_src),
            ]

            try:
                run_command(cmd)
            finally:
                if not args.keep_staging_dirs:
                    shutil.rmtree(staging_dir, ignore_errors=True)
    finally:
        if vendor_temp_root is not None and not args.keep_staging_dirs:
            shutil.rmtree(vendor_temp_root, ignore_errors=True)

    print("Finished staging npm packages.")
    return 0


def collect_native_components(packages: list[str]) -> set[str]:
    components: set[str] = set()
    for package in packages:
        components.update(PACKAGE_NATIVE_COMPONENTS.get(package, []))
    return components


def install_native_components(
    workflow_url: str,
    components: set[str],
    vendor_root: Path,
) -> None:
    if not components:
        return

    cmd = [str(INSTALL_NATIVE_DEPS), "--workflow-url", workflow_url, str(vendor_root)]
    run_command(cmd)


def run_command(cmd: list[str]) -> None:
    printable = " ".join(cmd)
    print("+", printable)
    subprocess.run(cmd, cwd=REPO_ROOT, check=True)


if __name__ == "__main__":
    raise SystemExit(main())
