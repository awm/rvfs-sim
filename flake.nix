# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        toolchain

        # We want the unwrapped version, "rust-analyzer" (wrapped) comes with nixpkgs' toolchain
        pkgs.rust-analyzer-unwrapped

        # Build requirements
        pkgs.cmake
        pkgs.openssl
        pkgs.pkg-config

        # Cargo packages
        pkgs.cargo-binutils
        pkgs.cargo-expand

        # Debugging and flashing tools
        pkgs.gdb
      ];

      RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    };
  };
}
