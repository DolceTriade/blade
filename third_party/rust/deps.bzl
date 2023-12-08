"""Third party rust dependencies."""

load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository", "render_config")

def rust_dependencies():
    crates_repository(
        name = "crate",
        cargo_lockfile = "//third_party/rust:Cargo.lock",
        lockfile = "//third_party/rust:Cargo.Bazel.lock",
        annotations = {
            "zstd-sys": [
                crate.annotation(
                    build_script_env = {
                        "AR": "external/bintools/bin/ar",
                    },
                    build_script_tools = [
                        "@bintools//:bin/ar",
                    ],
                ),
            ],
            "server_fn_macro": [
                crate.annotation(
                    rustc_env = {
                        "SERVER_FN_OVERRIDE_KEY": "bazel",
                    },
                ),
            ],
            "protoc-gen-prost": [crate.annotation(
                gen_binaries = ["protoc-gen-prost"],
            )],
            "protoc-gen-tonic": [crate.annotation(
                gen_binaries = ["protoc-gen-tonic"],
            )],
            "prost-build": [crate.annotation(
                patches = ["@//third_party/rust/patches/prost-build:0001-Allow-substitution-for-the-message-type-in-type-attr.patch"],
                patch_args = ["-p1"],
            )],
            "pq-sys": [
                crate.annotation(
                    build_script_env = {
                        "AR": "external/bintools/bin/ar",
                    },
                    build_script_tools = [
                        "@bintools//:bin/ar",
                    ],
                ),
            ],
            "libsqlite3-sys": [
                crate.annotation(
                    build_script_env = {
                        "AR": "external/bintools/bin/ar",
                    },
                    build_script_tools = [
                        "@bintools//:bin/ar",
                    ],
                ),
            ],
            "web-sys": [
                crate.annotation(
                    crate_features = ["DomRectList", "DomRect", "DomRectReadOnly", "DomQuad"],
                ),
            ],
        },
        packages = {
            "ansi-to-html": crate.spec(
                version = "0.1.3",
            ),
            "actix-files": crate.spec(
                version = "0.6",
            ),
            "actix-web": crate.spec(
                version = "4",
                features = ["macros"],
            ),
            "anyhow": crate.spec(
                version = "1.0.75",
            ),
            "async-stream": crate.spec(
                version = "0.3",
            ),
            "broadcaster": crate.spec(
                version = "1",
            ),
            "clap": crate.spec(
                version = "4.4.10",
                features = ["derive", "wrap_help"],
            ),
            "diesel": crate.spec(
                version = "2.1.4",
                features = ["extras", "sqlite", "postgres"],
            ),
            "futures": crate.spec(
                version = "0.3.29",
            ),
            "futures-core": crate.spec(
                version = "0.3.29",
            ),
            "cfg-if": crate.spec(
                version = "1",
            ),
            "lazy_static": crate.spec(
                version = "1",
            ),
            "leptos": crate.spec(
                version = "0.5.2",
                features = ["ssr"],
            ),
            "leptos_actix": crate.spec(
                version = "0.5.2",
            ),
            "leptos_meta": crate.spec(
                version = "0.5.2",
                features = ["ssr"],
            ),
            "leptos_router": crate.spec(
                version = "0.5.2",
                features = ["ssr"],
            ),
            "log": crate.spec(
                version = "0.4",
            ),
            "pretty_env_logger": crate.spec(
                version = "0.5.0",
            ),
            "serde": crate.spec(
                version = "1.0.186",
                features = ["derive"],
            ),
            "serde_json": crate.spec(
                version = "1.0.108",
            ),
            "prost": crate.spec(
                version = "0.12.3",
            ),
            "prost-types": crate.spec(
                version = "0.12.3",
            ),
            "protoc-gen-prost": crate.spec(
                version = "0.2.3",
            ),
            "protoc-gen-tonic": crate.spec(
                version = "0.3.0",
            ),
            "prost-reflect": crate.spec(
                version = "0.12.0",
                features = ["derive", "serde", "text-format"],
            ),
            "scopeguard": crate.spec(
                version = "1.2.0",
            ),
            "tokio": crate.spec(
                version = "1.32.0",
                features = ["full"],
            ),
            "tokio-stream": crate.spec(
                version = "0.1",
            ),
            "tonic": crate.spec(
                version = "0.10.2",
            ),
            "tonic-reflection": crate.spec(
                version = "0.10.2",
            ),
            "walkdir": crate.spec(
                version = "2.4.0",
            ),
        },
        # Setting the default package name to `""` forces the use of the macros defined in this repository
        # to always use the root package when looking for dependencies or aliases. This should be considered
        # optional as the repository also exposes alises for easy access to all dependencies.
        render_config = render_config(
            default_package_name = "",
        ),
        rust_toolchain_cargo_template = "@nix_rust//:bin/{tool}",
        rust_toolchain_rustc_template = "@nix_rust//:bin/{tool}",
        generator = "@cargo_bazel_bootstrap//:cargo-bazel",
    )
    crates_repository(
        name = "wasm_crate",
        cargo_lockfile = "//third_party/rust:WasmCargo.lock",
        lockfile = "//third_party/rust:WasmCargo.Bazel.lock",
        annotations = {
            "getrandom": [
                crate.annotation(
                    crate_features = ["js"],
                ),
            ],
            "server_fn_macro": [
                crate.annotation(
                    rustc_env = {
                        "SERVER_FN_OVERRIDE_KEY": "bazel",
                    },
                ),
            ],
            "web-sys": [
                crate.annotation(
                    crate_features = ["DomRectList", "DomRect", "DomRectReadOnly", "DomQuad"],
                ),
            ],
        },
        packages = {
            "ansi-to-html": crate.spec(
                version = "0.1.3",
            ),
            "leptos": crate.spec(
                version = "0.5.2",
                features = ["hydrate"],
            ),
            "leptos_meta": crate.spec(
                version = "0.5.2",
                features = ["hydrate"],
            ),
            "leptos_router": crate.spec(
                version = "0.5.2",
                features = ["hydrate"],
            ),
            "gloo-net": crate.spec(
                version = "0.4.0",
            ),
            "serde": crate.spec(
                version = "1.0.186",
                features = ["derive"],
            ),
            "log": crate.spec(
                version = "0.4",
            ),
            "futures-util": crate.spec(
                version = "0.3.29",
            ),
            "const_format_proc_macros": crate.spec(
                version = "0.2.31",
            ),
            "ahash": crate.spec(
                version = "0.7.6",
                default_features = False,
                features = ["std"],
            ),
            "wasm-bindgen": crate.spec(
                version = "0.2.87",
            ),
            "console_log": crate.spec(
                version = "1",
            ),
            "cfg-if": crate.spec(
                version = "1",
            ),
            "console_error_panic_hook": crate.spec(
                version = "0.1",
            ),
            "futures": crate.spec(
                version = "0.3.29",
            ),
            "futures-core": crate.spec(
                version = "0.3.29",
            ),
            "tokio": crate.spec(
                version = "1.32.0",
                features = ["full"],
            ),
            "tokio-stream": crate.spec(
                version = "0.1",
            ),
        },
        # Setting the default package name to `""` forces the use of the macros defined in this repository
        # to always use the root package when looking for dependencies or aliases. This should be considered
        # optional as the repository also exposes alises for easy access to all dependencies.
        render_config = render_config(
            default_package_name = "",
        ),
        rust_toolchain_cargo_template = "@nix_rust_wasm//:bin/{tool}",
        rust_toolchain_rustc_template = "@nix_rust_wasm//:bin/{tool}",
        generator = "@cargo_bazel_bootstrap//:cargo-bazel",
    )
