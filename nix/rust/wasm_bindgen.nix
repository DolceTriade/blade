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
    ];
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  wasm-bindgen-cli' = pkgs.wasm-bindgen-cli.override {
    inherit rustPlatform;
  };
in
  wasm-bindgen-cli'.overrideAttrs (self: super: rec {
    version = "0.2.100";
    src = with self;
      pkgs.fetchCrate {
        inherit pname;
        version = "0.2.100";
        hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
      };
    cargoDeps = super.cargoDeps.overrideAttrs (_: {
      inherit src;
      outputHash = "sha256-uLT2eN5HE7RGLOOD3d+y2TcyXFp8Ol02jGlJZvOvMq0=";
    });
    doCheck = false;
  })
