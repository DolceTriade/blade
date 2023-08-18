{
  description = "BLADE";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    devenv.url = "github:cachix/devenv/latest";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    ...
  } @ inputs: (flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {inherit system;};
      bazelEnv = import ./nix/bazel/env.nix {inherit pkgs;};
    in {
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
                cargo
                pkg-config
                rust-analyzer
                rustc
                wasm-bindgen-cli
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
