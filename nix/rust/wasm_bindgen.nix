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
    version = "0.2.87";
    src = with self;
      pkgs.fetchCrate {
        inherit pname;
        version = "0.2.87";
        hash = "sha256-0u9bl+FkXEK2b54n7/l9JOCtKo+pb42GF9E1EnAUQa0=";
      };
    cargoDeps = super.cargoDeps.overrideAttrs (_: {
      inherit src;
      outputHash = "sha256-9E37D3x2gB/b4+kgwS1FILCqbVLHpPuLW2s+FaS4J2c=";
    });
    doCheck = false;
  })
