{
  description = "A new FRAME-based Substrate node, ready for hacking.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "wasm32-unknown-unknown" ];
          });
      in with pkgs; {
        devShells.default = mkShell {
          buildInputs = [
            clang
            pkg-config

            rust

          ] ++ lib.optional stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.Security ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          PROTOC = "${protobuf}/bin/protoc";
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library/";
          ROCKSDB_LIB_DIR = "${rocksdb}/lib";
        };
      });
}
