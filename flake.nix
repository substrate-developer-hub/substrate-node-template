{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      toolchainToml = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
      toolchainBasedRust = pkgs.rust-bin."${toolchainToml.toolchain.channel}".latest."${toolchainToml.toolchain.profile}".override {
            extensions = toolchainToml.toolchain.components;
            inherit (toolchainToml.toolchain) targets;
      };
    in
    {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          toolchainBasedRust
          clang
          protobuf
        ];

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      };
    });
}
