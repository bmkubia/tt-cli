# Releasing tt-cli

Automation drives releases across GitHub, npm, and Homebrew. This
document captures the end-to-end flow so maintainers can cut alpha builds as
often as needed while keeping public releases deliberate.

## Distribution channels

- **GitHub Releases** – canonical binaries for every supported platform.
- **npm (`tt-cli`)** – wraps the Rust binary in a Node launcher (`npm install -g tt-cli`).
- **Homebrew cask** – uses the GitHub release tarballs; Homebrew automation usually
  opens a PR automatically, but we keep local tooling to help when manual bumps are required.

## Prerequisites

- `gh` CLI authenticated with write access to the repo.
- `git-cliff` installed locally if you want to preview notes (`cargo install git-cliff --locked`).
- Clean `master` branch.
- All quality gates pass locally: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.

## Cutting a release

1. Decide whether you are publishing an alpha (`--publish-alpha`) or a public
   release (`--publish-release`). Alpha releases advance the `-alpha.N`
   suffix for the upcoming minor version; stable releases bump the minor version.
2. Run the helper script from the repo root:

   ```bash
   ./scripts/create_github_release.py --dry-run --publish-alpha
   ./scripts/create_github_release.py --publish-alpha
   ./scripts/create_github_release.py --publish-release
   ```

   Use `TT_RELEASE_REPO=your-org/tt-cli` if you're running from a fork.
3. The script creates a synthetic commit that only bumps `Cargo.toml`, tags it as
   `tt-vX.Y.Z`, and pushes the tag. No commit lands on `master`, which keeps the
   working tree pinned at `0.0.0`.
4. The `release` workflow (see `.github/workflows/release.yml`) kicks off:
   - Validates the tag matches `Cargo.toml`
   - Runs fmt/clippy/test once
   - Builds release binaries for macOS, Linux (glibc + musl), and Windows
   - Generates git-cliff notes, stages npm tarballs, and creates the GitHub Release
   - Publishes `tt-cli` to npm (stable + `-alpha.N` tags only)

Monitor <https://github.com/bmkubia/tt-cli/actions/workflows/release.yml> for status. You can
re-run failed jobs directly from the Actions UI.

## npm publishing

The workflow stages npm packages via `scripts/stage_npm_packages.py`, which:

1. Downloads the workflow artifacts via `gh run download` (`scripts/install_native_deps.py`)
2. Hydrates the `vendor/` tree with all platform binaries
3. Calls `scripts/build_npm_package.py` to inject the version and run `npm pack`

The publish job reuses trusted publishing (`id-token: write`) so no long-lived
`NODE_AUTH_TOKEN` is required. If you need to re-publish locally, run:

```bash
node --version     # must be >= 18
python3 scripts/stage_npm_packages.py --release-version <version> \
  --package tt-cli \
  --workflow-url https://github.com/<org>/tt-cli/actions/runs/<run_id> \
  --output-dir dist/npm
cd dist/npm && npm publish tt-cli-<version>.tgz
```

## Homebrew

Homebrew's automation periodically scans GitHub Releases and usually opens a PR
against [`homebrew/homebrew-cask`](https://github.com/Homebrew/homebrew-cask).
When that does not happen fast enough, update the cask manually:

1. Download the macOS tarballs from the latest GitHub release (one per arch).
2. Run `scripts/homebrew_print_checksums.py --artifacts-dir <download-dir>` to print the
   `sha256` values expected by the cask.
3. Use `brew bump-cask-pr --cask tt` with the new version, URLs:
   `https://github.com/<org>/tt-cli/releases/download/tt-v<version>/tt-<target>.tar.gz`.

Document any manual adjustments in the PR. When the cask merges, `brew install --cask tt`
will deliver the notarized binaries.

## Verification checklist

- [ ] Workflow succeeds (lint + build matrix + release + npm publish)
- [ ] GitHub Release shows the git-cliff notes
- [ ] `npm info tt-cli version` reflects the new version (or `alpha` dist-tag)
- [ ] `brew info tt` lists the latest version once the cask PR merges
- [ ] README install instructions stay accurate (update if new channels are added)
