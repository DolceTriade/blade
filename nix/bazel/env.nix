{pkgs ? import <nixpkgs> {}}: let
  bazelEnv = with pkgs; [
    bash
    coreutils
    diffutils
    file
    findutils
    gawk
    gnugrep
    gnused
    gnutar
    gzip
    nix
    python3
    unzip
    which
    zip
    bintools
  ];
in
  pkgs.buildEnv {
    name = "bazel-env";
    paths = bazelEnv;
    pathsToLink = ["/bin"];
  }
