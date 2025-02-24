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
      targets.wasm32-unknown-unknown.latest.rust-std
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
      outputHash = "sha256-/8T0FGhPMQoUM5/M8lZkTGAc9ul+/Xe59xe0Z/l/RsI=";
    });
    doCheck = false;
  })
