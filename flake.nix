{
  description = "BLADE";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }: (flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      formatter.default = pkgs.alejandra;
      devShells.default = pkgs.mkShell {
        packages = with pkgs;
          [
            bazel_6
            bazel-buildtools
            bazel-watcher
            cargo
            pkg-config
            rust-analyzer
            rustc
          ]
          ++ pkgs.lib.optional pkgs.stdenv.isDarwin pkgs.darwin.cctools;
        shellhook = ''
          unalias ls
        '';
      };
    }));
}
