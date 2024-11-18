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
    version = "0.2.95";
    src = with self;
      pkgs.fetchCrate {
        inherit pname;
        version = "0.2.95";
        hash = "sha256-prMIreQeAcbJ8/g3+pMp1Wp9H5u+xLqxRxL+34hICss=";
      };
    cargoDeps = super.cargoDeps.overrideAttrs (_: {
      inherit src;
      outputHash = "sha256-00AJAu45ggJPm77CtExcQGU542oqx31kj0pBqeqMR+0=";
    });
    doCheck = false;
  })
