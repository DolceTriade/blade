{target ? ""}: let
  og = import <nixpkgs> {};
  fenix = import <fenix> {pkgs = og;};
  pkgs = import <nixpkgs> {
    overlays = [
      (self: super: {
        inherit fenix;
      })
    ];
  };
  lib = pkgs.lib;
  wasm = target == "wasm32-unknown-unknown";
  rust = with pkgs.fenix;
  with latest;
    combine ([
      cargo
      clippy
      rust-src
      rustc
      rustfmt
      rust-analyzer
    ] ++ lib.optional wasm targets.wasm32-unknown-unknown.latest.rust-std);
  os = pkgs.stdenv.targetPlatform.rust.platform.os;
  build-triple = pkgs.stdenv.buildPlatform.rust.rustcTargetSpec;
  target-triple =
    if target != ""
    then target
    else pkgs.stdenv.targetPlatform.rust.rustcTargetSpec;
  binary-ext = lib.optionalString wasm ".wasm";
  staticlib-ext = ".a";
  dylib-ext =
    if wasm
    then ".wasm"
    else if os == "macos"
    then ".dylib"
    else ".so";
in
  pkgs.buildEnv {
    extraOutputsToInstall = ["out"];
    name = "bazel-rust-toolchain";
    paths = [rust.out];
    pathsToLink = ["/bin" "/etc" "/lib" "/libexec" "rustc_src" "/share"];
    postBuild = ''
      cat <<EOF > $out/BUILD
      filegroup(
          name = "rustc",
          srcs = ["bin/rustc"],
          visibility = ["//visibility:public"],
      )

      filegroup(
          name = "rustfmt",
          srcs = ["bin/rustfmt"],
          visibility = ["//visibility:public"],
      )

      filegroup(
          name = "cargo",
          srcs = ["bin/cargo"],
          visibility = ["//visibility:public"],
      )

      filegroup(
          name = "clippy_driver",
          srcs = ["bin/clippy-driver"],
          visibility = ["//visibility:public"],
      )

      filegroup(
          name = "rustc_lib",
          srcs = glob(
              [
                  "bin/*.so",
                  "lib/*.so",
                  "lib/rustlib/*/codegen-backends/*.so",
                  "lib/rustlib/*/codegen-backends/*.dylib",
                  "lib/rustlib/*/bin/rust-lld",
                  "lib/rustlib/*/lib/*.so",
                  "lib/rustlib/*/lib/*.dylib",
              ],
              allow_empty = True,
          ),
          visibility = ["//visibility:public"],
      )

      filegroup(
        name = "rust-analyzer",
        srcs = ["bin/rust-analyzer"],
        visibility = ["//visibility:public"],
      )

      filegroup(
        name = "rust-analyzer-proc-macro-srv",
        srcs = ["libexec/rust-analyzer-proc-macro-srv"],
        visibility = ["//visibility:public"],
      )

      load("@rules_rust//rust:toolchain.bzl", "rust_stdlib_filegroup")
      rust_stdlib_filegroup(
          name = "rust_std",
          srcs = glob(
              [
                  "lib/rustlib/*/lib/*.rlib",
                  "lib/rustlib/*/lib/*.so",
                  "lib/rustlib/*/lib/*.dylib",
                  "lib/rustlib/*/lib/*.a",
                  "lib/rustlib/*/lib/self-contained/**",
              ],
              # Some patterns (e.g. `lib/*.a`) don't match anything, see https://github.com/bazelbuild/rules_rust/pull/245
              allow_empty = True,
          ),
          visibility = ["//visibility:public"],
      )

      filegroup(
          name = "rust_doc",
          srcs = ["bin/rustdoc"],
          visibility = ["//visibility:public"],
      )

      load('@rules_rust//rust:toolchain.bzl', 'rust_toolchain')
      rust_toolchain(
          name = "rust_nix_impl",
          rust_doc = ":rust_doc",
          rust_std = ":rust_std",
          rustc = ":rustc",
          rustfmt = ":rustfmt",
          cargo = ":cargo",
          clippy_driver = ":clippy_driver",
          rustc_lib = ":rustc_lib",
          binary_ext = "${binary-ext}",
          staticlib_ext = "${staticlib-ext}",
          dylib_ext = "${dylib-ext}",
          exec_triple = "${build-triple}",
          target_triple = "${target-triple}",
          default_edition = "2024",
          stdlib_linkflags = ["-lpthread", "-ldl"],
          visibility = ["//visibility:public"],
      )
      EOF
      mkdir $out/rustc_src
      (cd $out/rustc_src && ln -s ../lib/rustlib/src/rust/library .)
      cat > $out/rustc_src/BUILD <<EOF
      filegroup(
        name = "rustc_src",
        srcs = glob(
          [
            "*/**",
          ],
          allow_empty = True,
        ),
        visibility = ["//visibility:public"],
      )
      EOF
    '';
  }
