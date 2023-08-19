{target ? ""}: let
  og = import <nixpkgs> {};
  fenix = import <fenix> {pkgs = og;};
  pkgs = import <nixpkgs> {
    system = builtins.currentSystem;
    overlays = [
      (self: super: {
        inherit fenix;
      })
    ];
  };
  rust = with pkgs.fenix;
  with stable;
    combine [
      cargo
      clippy
      rust-src
      rustc
      rustfmt
      targets.wasm32-unknown-unknown.stable.rust-std
    ];
  ogRust = pkgs.rust;
  os = ogRust.toTargetOs pkgs.stdenv.targetPlatform;
  build-triple = ogRust.toRustTargetSpec pkgs.stdenv.buildPlatform;
  target-triple =
    if target != ""
    then target
    else ogRust.toRustTargetSpec pkgs.stdenv.targetPlatform;
  binary-ext = "";
  staticlib-ext = ".a";
  dylib-ext =
    if os == "macos"
    then ".dylib"
    else ".so";
in
  pkgs.buildEnv {
    extraOutputsToInstall = ["out"];
    name = "bazel-rust-toolchain";
    paths = [rust.out];
    pathsToLink = ["/bin" "/etc" "/lib" "/libexec" "/share"];
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
          default_edition = "2021",
          stdlib_linkflags = ["-lpthread", "-ldl"],
          visibility = ["//visibility:public"],
      )
      EOF
    '';
  }
