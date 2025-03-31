let
  pkgs = import <nixpkgs> {};
in
  pkgs.callPackage ./default.nix {
    libunwind = pkgs.pkgsStatic.libunwind;
  }
