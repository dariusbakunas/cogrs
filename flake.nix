{
  inputs = {
    nixpkgs = {url = "github:NixOS/nixpkgs/nixos-unstable";};
    flake-utils = {url = "github:numtide/flake-utils";};
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        # this is how we can tell crane to use our toolchain!
        craneLibWithoutRegistry = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        shipyardToken = builtins.readFile ./secrets/shipyard-token;
        craneLib = craneLibWithoutRegistry.appendCrateRegistries [
          (craneLibWithoutRegistry.registryFromDownloadUrl {
            dl = "https://crates.shipyard.rs/api/v1/crates";
            indexUrl = "ssh://git@ssh.shipyard.rs/catscii/crate-index.git";
            fetchurlExtraArgs = {
              curlOptsList = ["-H" "user-agent: shipyard ${shipyardToken}"];
            };
          })
        ];
        src = craneLib.cleanCargoSource ./.;
        # as before
        nativeBuildInputs = with pkgs; [rustToolchain pkg-config];
        buildInputs = with pkgs; [openssl sqlite];
        # because we'll use it for both `cargoArtifacts` and `bin`
        commonArgs = {
          inherit src buildInputs nativeBuildInputs;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        # remember, `set1 // set2` does a shallow merge:
        bin = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
          });
      in
        with pkgs; {
          packages = {
            # that way we can build `bin` specifically,
            # but it's also the default.
            inherit bin;
            default = bin;
          };
          devShells.default = mkShell {
            # instead of passing `buildInputs` / `nativeBuildInputs`,
            # we refer to an existing derivation here
            inputsFrom = [bin];
          };
        }
    );
}
