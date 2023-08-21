{
  description = "BLADE";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    devenv.url = "github:cachix/devenv/latest";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    fenix,
    ...
  } @ inputs: (flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [fenix.overlays.default];
      };
      bazelEnv = import ./nix/bazel/env.nix {inherit pkgs;};
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
    in {
      packages.rust = rust;
      formatter.default = pkgs.alejandra;
      devShells.default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ({pkgs, ...}: {
            packages = with pkgs;
              [
                bazel_6
                bazel-buildtools
                bazel-watcher
                pkg-config
                rust
                grpcurl
                rnix-lsp
              ]
              ++ pkgs.lib.optional pkgs.stdenv.isDarwin pkgs.darwin.cctools;
            enterShell = ''
              echo "BLADE Shell"
              echo "build --action_env=PATH=${bazelEnv}/bin" > .bazelenvrc
            '';
          })
        ];
      };
    }));
}
