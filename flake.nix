{
  description = "Hexagonal Chess WASM Game with P2P Multiplayer";
  nixConfig = {
    substituters = [
      "https://cache.nixos.org/"
      "https://nix-community.cachix.org"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
    # optional: quiet the buffer warning
    download-buffer-size = 268435456; # 256MiB
  };
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { 
          inherit system; 
          overlays = [ 
            rust-overlay.overlays.default
            # (final: prev: {
            #   # Override nodejs to skip tests (some tests timeout on macOS)
            #   nodejs = prev.nodejs.overrideAttrs (oldAttrs: {
            #     doCheck = false;
            #   });
            # })
          ]; 
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustPlatform = pkgs.makeRustPlatform { 
          cargo = rustToolchain; 
          rustc = rustToolchain; 
        };
      in {
        packages.core = rustPlatform.buildRustPackage {
          pname = "hex-chess-core";
          version = "0.1.0";
          src = ./crates/core;
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = false;
        };

        packages.game = pkgs.stdenv.mkDerivation {
          pname = "hex-chess-game";
          version = "0.1.0";
          src = self;
          buildInputs = [ pkgs.wasm-bindgen-cli pkgs.trunk ];
          buildPhase = ''
            cd crates/game
            trunk build --release
          '';
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };

        packages.signaling = rustPlatform.buildRustPackage {
          pname = "hex-chess-signaling";
          version = "0.1.0";
          src = ./crates/signaling;
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = false;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.cargo
            pkgs.wasm-bindgen-cli
            pkgs.trunk
            pkgs.clang
            pkgs.lld
            pkgs.pkg-config
            pkgs.openssl.dev
            pkgs.nodejs_20
          ];
          shellHook = ''
            # Add lld to PATH for WASM linking
            export PATH="${pkgs.lld}/bin:$PATH"
          '';
        };
      });
}
