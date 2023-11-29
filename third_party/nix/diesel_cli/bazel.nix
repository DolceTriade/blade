let
  platform = import ../../../nix/rust/rust_platform.nix;
  pkgs = platform.pkgs;
in
  with platform;
    pkgs.callPackage ./default.nix {
      inherit rustPlatform;
      diesel-cli = pkgs.diesel-cli;
    }
