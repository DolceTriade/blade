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

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };
      leptosfmt = pkgs.callPackage ./third_party/nix/leptosfmt {inherit rustPlatform;};
      diesel-cli' = pkgs.callPackage ./third_party/nix/diesel_cli {inherit rustPlatform;};
    in {
      packages.rust = rust;
      formatter.default = pkgs.alejandra;
      devShells.default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ({pkgs, ...}: {
            packages = with pkgs;
              [
                alejandra
                bazel_6
                bazel-buildtools
                pkg-config
                rust
                grpcurl
                leptosfmt
                diesel-cli'
                wabt
                (import ./nix/cc/cc.nix)
              ]
              ++ pkgs.lib.optional pkgs.stdenv.isDarwin pkgs.darwin.cctools;
            enterShell = ''
              echo "BLADE Shell"
              echo "build --action_env=PATH=${bazelEnv}/bin" > .bazelenvrc
              echo "build --host_action_env=PATH=${bazelEnv}/bin" >> .bazelenvrc
            '';
          })
        ];
      };
    }));
}
