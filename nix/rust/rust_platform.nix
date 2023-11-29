let
  og = import <nixpkgs> {};
  fenix = import <fenix> {pkgs = og;};
  pkgs = import <nixpkgs> {
    system = builtins.currentSystem;
    overlays = [
      (self: super: {
        inherit fenix;
      })
    ];
  };
  rust = with pkgs.fenix;
  with stable;
    combine [
      cargo
      clippy
      rust-src
      rustc
      rustfmt
      targets.wasm32-unknown-unknown.stable.rust-std
      rust-analyzer
    ];
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  in {
    inherit pkgs rustPlatform;
  }
