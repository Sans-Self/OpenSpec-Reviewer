## 1. Release metadata

- [x] 1.1 Set the crate version to 1.0.1 and add its repository URL.
- [x] 1.2 Pin cargo-dist and the four supported release targets in repository metadata.

## 2. Release automation

- [x] 2.1 Generate the GitHub release workflow with archives, checksums, a shell installer, and artifact attestations.
- [x] 2.2 Keep the Nix build and Cachix workflow independent from native release assets.

## 3. Installation guidance

- [x] 3.1 Document the shell installer, direct release downloads, and the `git` and `gh` runtime expectations.
- [x] 3.2 Document Nix installation without overriding the reviewer's locked inputs.

## 4. Verification

- [x] 4.1 Validate the OpenSpec change and review it with `openspec-reviewer`.
- [x] 4.2 Run formatting, tests, Clippy, the cargo-dist plan, and the Nix build.
- [x] 4.3 Publish v1.0.1 and verify its archives, checksums, installer, and provenance.
