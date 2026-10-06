## Context

The v1.0.0 tag has a signed Nix store path in `sans-self.cachix.org`, but its GitHub release has no assets. Nix cannot silently trust a third-party cache on every installation, and overriding the flake's `nixpkgs` input changes the derivation that CI cached.

The executable is one Rust binary. It shells out to `git` and optionally `gh`; those programs remain runtime prerequisites for the corresponding commands.

## Goals / Non-Goals

**Goals:**

- Give macOS and Linux users archives built from the tagged source.
- Give users one shell installer that selects a supported archive and verifies its checksum.
- Publish provenance with the release assets.
- Preserve the flake and Cachix path for Nix users.

**Non-Goals:**

- Windows packaging.
- Automatic updates inside the reviewer.
- Apple notarisation or operating-system package signing.
- Publishing `git`, `gh`, or OpenSpec with the binary.

## Decisions

Use cargo-dist 0.33.0 to generate the GitHub release workflow. It builds versioned archives, checksums, the shell installer, and GitHub artifact attestations from the version tag. Pinning the generator version makes changes to release automation explicit.

Build `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu`, and `x86_64-unknown-linux-gnu`. These targets cover the current Nix systems while adding Intel macOS and Arm Linux downloads.

Keep `.github/workflows/nix.yml` as the source-build, test, and Cachix workflow. The release workflow builds standalone Cargo binaries because a Nix-built macOS executable refers to libraries in `/nix/store`.

Treat the package version and version tag as one release identity. The workflow only publishes when the tag names the version in `Cargo.toml`.

Document release archives before Nix. The Nix example keeps the reviewer's own locked inputs so it can match the derivation uploaded to Cachix.

## Risks / Trade-offs

- Cross-compiled Arm Linux assets may fail in CI -> cargo-dist's generated workflow owns the target toolchain, and the tag is created only after its plan succeeds locally.
- A tag can trigger publication before every target completes -> cargo-dist stages workflow artifacts and creates the release after the build jobs succeed.
- GitHub-hosted artifacts make GitHub an installation dependency -> checksums and attestations make downloaded bytes verifiable; Nix remains an independent build path.
- The installer cannot supply `git` or `gh` -> README names those runtime requirements next to installation.
