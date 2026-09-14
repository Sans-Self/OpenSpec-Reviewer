{
  description = "openspec-reviewer — a terminal reviewer for OpenSpec changes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" ];
      forAll = f:
        nixpkgs.lib.genAttrs systems (system:
          f (import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          }));
      # OpenSpec CLI for openspec/ specs and changes. Not in nixpkgs; a
      # version-pinned dlx wrapper gives every shell a bare `openspec`
      # without making Node part of the project. First run downloads into
      # pnpm's dlx cache; offline after.
      openspecCli = pkgs: pkgs.writeShellScriptBin "openspec" ''
        exec ${pkgs.pnpm}/bin/pnpm --silent dlx @fission-ai/openspec@1.6.0 "$@"
      '';
    in
    {
      packages = forAll (pkgs:
        let
          rust = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform { cargo = rust; rustc = rust; };
          reviewer = rustPlatform.buildRustPackage {
            pname = "openspec-reviewer";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            # The source tests build throwaway repositories with git.
            nativeCheckInputs = [ pkgs.git ];
            preCheck = "export HOME=$TMPDIR";
            meta.mainProgram = "openspec-reviewer";
          };
        in
        {
          inherit reviewer;
          default = reviewer;
        });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })
            pkgs.gh
            pkgs.git
            (openspecCli pkgs)
          ];
        };
      });

      formatter = forAll (pkgs: pkgs.nixpkgs-fmt);
    };
}
