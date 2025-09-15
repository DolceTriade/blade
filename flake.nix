{
  description = "BLADE";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      with latest;
        combine [
          cargo
          clippy
          rust-src
          rustc
          rustfmt
          targets.wasm32-unknown-unknown.latest.rust-std
          rust-analyzer
        ];

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };
      leptosfmt = pkgs.callPackage ./third_party/nix/leptosfmt {inherit rustPlatform;};
      diesel-cli' = pkgs.callPackage ./third_party/nix/diesel_cli {
        inherit rustPlatform;
        mysqlSupport = false;
      };
      jemalloc' = pkgs.callPackage ./third_party/nix/jemalloc {};
      ibazel = pkgs.writeShellScriptBin "ibazel" ''
        ${pkgs.bazel-watcher}/bin/ibazel -bazel_path ${pkgs.bazel_7}/bin/bazel "$@"
      '';
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
                bazel_7
                ibazel
                bazel-buildtools
                pkg-config
                rust
                grpcurl
                git
                leptosfmt
                jemalloc'
                diesel-cli'
                wabt
                postgresql
                flamegraph
                tokio-console
                (import ./nix/cc/cc.nix {inherit pkgs;})
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
