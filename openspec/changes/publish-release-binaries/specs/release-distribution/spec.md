## ADDED Requirements

### Requirement: A version tag publishes native archives
The release workflow MUST publish an archive for every supported platform from the commit named by a version tag.

#### Scenario: All supported platforms build
- **WHEN** a version tag matching `Cargo.toml` reaches GitHub
- **THEN** the release contains an archive for `aarch64-apple-darwin`
- **AND** the release contains an archive for `x86_64-apple-darwin`
- **AND** the release contains an archive for `aarch64-unknown-linux-gnu`
- **AND** the release contains an archive for `x86_64-unknown-linux-gnu`

### Requirement: Every archive is verifiable
The release workflow MUST publish a SHA-256 checksum and GitHub build provenance for each native archive.

#### Scenario: A user downloads an archive
- **WHEN** the user verifies the archive against its release metadata
- **THEN** the checksum identifies the downloaded bytes
- **AND** the attestation identifies the repository and release workflow that built them

### Requirement: The shell installer selects a supported archive
The release workflow MUST publish a shell installer that maps the host operating system and architecture to one supported archive.

#### Scenario: The host is supported
- **GIVEN** the host matches one supported platform
- **WHEN** the user runs the release's shell installer
- **THEN** the installer downloads the archive for that platform
- **AND** the installer installs `openspec-reviewer` on the user's path

#### Scenario: The host is not supported
- **GIVEN** the host matches no supported platform
- **WHEN** the user runs the release's shell installer
- **THEN** the installer exits without installing a different platform's binary

### Requirement: Nix installation preserves release inputs
The documented flake input MUST retain the reviewer's locked `nixpkgs` and Rust overlay revisions.

#### Scenario: A project adds the reviewer flake
- **WHEN** the project evaluates the documented input
- **THEN** it does not replace the reviewer's locked inputs with project inputs
