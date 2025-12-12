{
  description = "nmrs development flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk/master";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        naersk-package = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
          clippy = toolchain;
        };
      in {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              cargo-info
              rustc
              rustfmt
              clippy
              rust-analyzer
              just

              eza
              fd
              fzf
              bat

              pkg-config
              libxkbcommon
              glib
              gobject-introspection
              gtk4
              libadwaita
            ];
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };

        packages.default = naersk-package.buildPackage {
          pname = "nmrs";
          src = ./.;

          buildInputs = with pkgs; [
            pkg-config
            wrapGAppsHook4
            libxkbcommon
            glib
          ];

          meta.mainProgram = "nmrs-gui";
        };
      }
    );
}
