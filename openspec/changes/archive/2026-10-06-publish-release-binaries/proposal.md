## Why

Users currently need Nix and explicit trust in a third-party binary cache, or they compile the reviewer from source. Tagged releases need ordinary downloadable binaries so installation does not depend on a package manager's trust configuration.

## What Changes

- Publish native macOS and Linux archives, checksums, a shell installer, and build provenance from every version tag.
- Build release assets from the tagged source on GitHub-hosted runners.
- Keep the Nix flake and Cachix as an additional installation path without rewriting the reviewer's locked inputs.
- Document binary installation first and direct Nix installation separately.

The first release does not add Windows packages, automatic self-update, or operating-system signing and notarisation.

## Capabilities

### New Capabilities

- `release-distribution`: Versioned tags produce verifiable native archives and an installer for supported systems.

### Modified Capabilities

None.

## Impact

The change adds cargo-dist configuration and a release workflow under `.github/workflows/`. It updates `Cargo.toml`, `Cargo.lock`, and `README.md`. GitHub Releases becomes the primary download host; Cachix remains available to Nix users. The separate Nixpkgs contribution consumes the same tagged source but does not live in this repository.
