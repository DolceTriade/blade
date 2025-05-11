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
  with latest;
    combine [
      cargo
      clippy
      rust-src
      rustc
      rustfmt
      rust-analyzer
    ];
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in {
  inherit pkgs rustPlatform;
}
