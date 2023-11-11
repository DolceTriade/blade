"""Nix deps."""

load("@io_tweag_rules_nixpkgs//nixpkgs:nixpkgs.bzl", "nixpkgs_package")

def third_party_nix_deps():
    nixpkgs_package(
        name = "bintools",
        repository = "@nixpkgs",
    )
    nixpkgs_package(
        name = "wasm-bindgen-cli",
        build_file = "//third_party/nix:BUILD.wasm-bindgen-cli",
        repositories = {
            "nixpkgs": "@nixpkgs",
            "fenix": "@fenix",
        },
        nix_file = "//nix/rust:wasm_bindgen.nix",
    )
    nixpkgs_package(
        name = "zlib",
        repository = "@nixpkgs",
    )
