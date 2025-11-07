# tt CLI (npm bundle)

This directory packages the Rust `tt` binary for npm distribution. The Node.js
launcher delegates to platform-specific binaries that live under `vendor/`.

Publishers should not edit the staged artifacts directly. Instead, run
`scripts/build_npm_package.py` from the repository root, which copies the files
in this directory, injects the correct version, bundles the platform binaries,
and emits a tarball suitable for `npm publish`.
