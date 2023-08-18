"""Nix deps."""

load("@io_tweag_rules_nixpkgs//nixpkgs:nixpkgs.bzl", "nixpkgs_package")

def third_party_nix_deps():
    nixpkgs_package(
        name = "cctools",
        nix_file_content = "let pkgs = import <nixpkgs>{}; in if pkgs.stdenv.isDarwin then pkgs.darwin.cctools else pkgs.cctools",
        repository = "@nixpkgs",
    )
    nixpkgs_package(
        name = "wasm-bindgen-cli",
        build_file = "//third_party/nix:BUILD.wasm-bindgen-cli",
        repository = "@nixpkgs",
    )
    nixpkgs_package(
        name = "zlib",
        repository = "@nixpkgs",
    )
