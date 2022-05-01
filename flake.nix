{
  description = "Keyboard";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    flake-utils.url = github:numtide/flake-utils;
    rust-overlay = {
      url = github:oxalica/rust-overlay;
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay } @ args: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          llvmPackages_latest.bintools
          picotool
          elf2uf2-rs
          probe-run
          ripgrep
          cargo-embed
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "thumbv6m-none-eabi" ];
          }))
        ];
      };
    }
  );
}
