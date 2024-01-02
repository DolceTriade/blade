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
    nixpkgs_package(
        name = "tailwindcss",
        attribute_path = "nodePackages.tailwindcss",
        repository = "@nixpkgs",
        build_file = "//third_party/nix:BUILD.tailwindcss",
    )
    nixpkgs_package(
        name = "diesel",
        repositories = {
            "nixpkgs": "@nixpkgs",
            "fenix": "@fenix",
        },
        nix_file = "//third_party/nix/diesel_cli:bazel.nix",
        nix_file_deps = [
            "//third_party/nix/diesel_cli:default.nix",
            "//nix/rust:rust_platform.nix",
        ],
    )
    nixpkgs_package(
        name = "sqlite",
        attribute_path = "sqlite.out",
        repository = "@nixpkgs",
        build_file = "//third_party/nix:BUILD.sqlite",
    )
    nixpkgs_package(
        name = "postgresql",
        attribute_path = "postgresql.lib",
        repository = "@nixpkgs",
        build_file = "//third_party/nix:BUILD.postgresql",
    )
    nixpkgs_package(
        name = "postgresql-bin",
        attribute_path = "postgresql",
        repository = "@nixpkgs",
    )