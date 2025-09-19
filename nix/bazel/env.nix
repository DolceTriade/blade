{pkgs ? import <nixpkgs> {}}: let
  bazelEnv = with pkgs;
    [
      bash
      coreutils
      diffutils
      file
      findutils
      gawk
      gnugrep
      gnumake
      gnused
      gnutar
      gzip
      nix
      python3
      unzip
      which
      zip
      bintools
      (import ../cc/cc.nix {inherit pkgs;})
    ]
    ++ (
      if pkgs.stdenv.isDarwin
      then [pkgs.apple-sdk pkgs.xcbuild]
      else []
    );
in
  pkgs.buildEnv {
    name = "bazel-env";
    paths = bazelEnv;
    pathsToLink = ["/bin"];
  }
