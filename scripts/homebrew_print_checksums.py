#!/usr/bin/env python3
"""Compute SHA256 checksums for macOS release tarballs to help with cask updates."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

MAC_TARGETS = {
    "aarch64-apple-darwin": "arm64",
    "x86_64-apple-darwin": "intel",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--artifacts-dir",
        type=Path,
        default=Path("dist"),
        help="Directory containing per-target artifact folders (default: dist).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    artifacts_dir = args.artifacts_dir.resolve()
    results = {}

    for target, label in MAC_TARGETS.items():
        tarball = artifacts_dir / target / f"tt-{target}.tar.gz"
        if not tarball.exists():
            raise SystemExit(f"Missing artifact: {tarball}")
        results[label] = sha256sum(tarball)

    print("Homebrew cask sha256 stanzas:")
    for label, digest in results.items():
        print(f'  {label}: "{digest}"')
    return 0


def sha256sum(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(8192), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


if __name__ == "__main__":
    raise SystemExit(main())
