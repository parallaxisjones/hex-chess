{
  description = "Hexagonal Chess WASM Game with P2P Multiplayer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
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
            pkgs.pkg-config
            pkgs.openssl.dev
            pkgs.nodejs
            pkgs.nodePackages.npm
          ];
        };
      });
}
