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
            "diesel": [
                crate.annotation(
                    deps = [
                        "@postgresql",
                        "@sqlite",
                    ],
                ),
            ],
            "linux-raw-sys": [
                crate.annotation(
                    crate_features = ["prctl"],
                ),
            ],
            "rustix": [
                crate.annotation(
                    crate_features = ["process"],
                ),
            ],
            "wasm-bindgen": [
                crate.annotation(
                    version = "=0.2.100",
                ),
            ],
            "tikv-jemalloc-sys": [
                crate.annotation(
                    build_script_data = ["@jemalloc//:lib/libjemalloc_pic.a"],
                    deps = ["@jemalloc"],
                    build_script_env = {
                        "JEMALLOC_OVERRIDE": "$(location @jemalloc//:lib/libjemalloc_pic.a)",
                    },
                ),
            ],
            "tokio": [
                crate.annotation(
                    rustc_flags = ["--cfg", "tokio_unstable"],
                    deps = ["@crate//:tracing"],
                ),
            ],
            "console-subscriber": [
                crate.annotation(
                    rustc_flags = ["--cfg", "tokio_unstable"],
                ),
            ],
            "unwind-sys": [
                crate.annotation(
                    build_script_data = ["@libunwind//:pkgconfig", "@pkg-config//:bin/pkg-config"],
                    build_script_env = {
                        "PKG_CONFIG": "$(location @pkg-config//:bin/pkg-config)",
                        "PKG_CONFIG_PATH": "$(location @libunwind//:pkgconfig)",
                    },
                ),
            ],
        },
        packages = {
            "ahash": crate.spec(
                version = "0.8.11",
            ),
            "ansi-to-html": crate.spec(
                version = "0.1.3",
            ),
            "actix-files": crate.spec(
                version = "0.6.5",
            ),
            "actix-web": crate.spec(
                version = "4.5.1",
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
                version = "4.5.32",
                features = ["derive", "wrap_help"],
            ),
            "derivative": crate.spec(
                version = "2.2.0",
            ),
            "diesel": crate.spec(
                version = "2.2.11",
                features = ["extras", "sqlite", "postgres", "returning_clauses_for_sqlite_3_35", "r2d2"],
            ),
            "diesel_migrations": crate.spec(
                version = "2.2.0",
                features = ["sqlite", "postgres"],
            ),
            "diesel-tracing": crate.spec(
                version = "0.3.1",
                features = ["sqlite", "postgres", "r2d2"],
            ),
            "futures": crate.spec(
                version = "0.3.29",
            ),
            "futures-core": crate.spec(
                version = "0.3.29",
            ),
            "flate2": crate.spec(
                version = "1.0",
            ),
            "cfg-if": crate.spec(
                version = "1",
            ),
            "humantime": crate.spec(
                version = "2.1.0",
            ),
            "junit-parser": crate.spec(
                version = "1.4.0",
                features = ["serde"],
            ),
            "lazy_static": crate.spec(
                version = "1",
            ),
            "leptos": crate.spec(
                version = "0.8.9",
                features = ["ssr", "nightly", "tracing", "trace-component-props"],
            ),
            "leptos_actix": crate.spec(
                version = "0.8.5",
                features = ["tracing"],
            ),
            "leptos_meta": crate.spec(
                version = "0.8.5",
                features = ["ssr", "tracing"],
            ),
            "leptos_router": crate.spec(
                version = "0.8.7",
                features = ["ssr", "tracing"],
            ),
            "leptos_dom": crate.spec(
                version = "0.8.6",
                features = ["tracing"],
            ),
            "log": crate.spec(
                version = "0.4",
            ),
            "serde": crate.spec(
                version = "1.0.186",
                features = ["derive"],
            ),
            "serde_json": crate.spec(
                version = "1.0.108",
            ),
            "prometheus-client": crate.spec(
                version = "0.22.2",
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
            "r2d2": crate.spec(
                version = "0.8.10",
            ),
            "regex": crate.spec(
                version = "1.11.1",
            ),
            "rstack-self": crate.spec(
                version = "0.3.0",
            ),
            "scopeguard": crate.spec(
                version = "1.2.0",
            ),
            "tempdir": crate.spec(
                version = "0.3.7",
            ),
            "tikv-jemallocator": crate.spec(
                version = "0.6.0",
                features = ["profiling", "stats", "unprefixed_malloc_on_supported_platforms"],
            ),
            "tikv-jemalloc-ctl": crate.spec(
                version = "0.6.0",
                features = ["profiling", "stats", "use_std"],
            ),
            "time": crate.spec(
                version = "0.3.37",
                features = ["formatting", "macros", "serde", "parsing"],
            ),
            "tokio": crate.spec(
                version = "1.32.0",
                features = ["full", "tracing"],
            ),
            "tokio-stream": crate.spec(
                version = "0.1",
            ),
            "console-subscriber": crate.spec(
                version = "0.4.1",
            ),
            "tonic": crate.spec(
                version = "0.10.2",
            ),
            "tonic-reflection": crate.spec(
                version = "0.10.2",
            ),
            "tracing": crate.spec(
                version = "0.1.41",
            ),
            "tracing-subscriber": crate.spec(
                version = "0.3.18",
                features = ["env-filter", "parking_lot", "json"],
            ),
            "tracing-actix-web": crate.spec(
                version = "0.7.9",
            ),
            "tracing-flame": crate.spec(
                version = "0.2.0",
            ),
            "url": crate.spec(
                version = "2.5.0",
            ),
            "url-escape": crate.spec(
                version = "0.1.1",
            ),
            "uuid": crate.spec(
                version = "1.6.1",
                features = [
                    "v5",  # SHA-1 based UUIDs
                    "v4",  # Lets you generate random UUIDs
                    "fast-rng",  # Use a faster (but still sufficiently random) RNG
                    "macro-diagnostics",  # Enable better diagnostics for compile-time UUIDs]
                ],
            ),
            "walkdir": crate.spec(
                version = "2.4.0",
            ),
            "wasm-bindgen": crate.spec(
                version = "=0.2.100",
            ),
            "web-sys": crate.spec(
                version = "0.3.77",
                features = ["Clipboard", "DomRectList", "DomRect", "DomRectReadOnly", "DomQuad", "File", "Url", "Blob", "BlobPropertyBag", "MediaQueryList", "Navigator", "Storage", "Window", "Performance", "HtmlCanvasElement", "CanvasRenderingContext2d", "Element", "HtmlElement", "CssStyleDeclaration"],
            ),
            "zip": crate.spec(
                version = "0.6.6",
                default_features = False,
                features = ["deflate", "time"],
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
        generator = "@cargo-bazel//:bin/cargo-bazel",
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
            "quick-xml": [
                crate.annotation(
                    rustc_flags = ["-O"],
                ),
            ],
            "wasm-bindgen": [
                crate.annotation(
                    version = "=0.2.100",
                ),
            ],
        },
        packages = {
            "ansi-to-html": crate.spec(
                version = "0.1.3",
            ),
            "anyhow": crate.spec(
                version = "1.0.75",
            ),
            "leptos": crate.spec(
                version = "0.8.9",
                features = ["hydrate", "tracing", "nightly", "trace-component-props"],
            ),
            "leptos_meta": crate.spec(
                version = "0.8.5",
                features = ["tracing"],
            ),
            "leptos_router": crate.spec(
                version = "0.8.7",
                features = ["tracing"],
            ),
            "leptos_dom": crate.spec(
                version = "0.8.6",
                features = ["tracing"],
            ),
            "gloo-file": crate.spec(
                version = "0.3.0",
            ),
            "gloo-net": crate.spec(
                version = "0.4.0",
            ),
            "serde": crate.spec(
                version = "1.0.186",
                features = ["derive"],
            ),
            "serde_json": crate.spec(
                version = "1.0.108",
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
                version = "0.8.11",
                default_features = False,
                features = ["std"],
            ),
            "wasm-bindgen": crate.spec(
                version = "=0.2.100",
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
            "flate2": crate.spec(
                version = "1.0",
            ),
            "junit-parser": crate.spec(
                version = "1.4.0",
                features = ["serde"],
            ),
            "humantime": crate.spec(
                version = "2.1.0",
            ),
            "time": crate.spec(
                version = "0.3.37",
                features = ["formatting", "macros", "parsing"],
            ),
            "tokio": crate.spec(
                version = "1.32.0",
                features = ["full"],
            ),
            "tokio-stream": crate.spec(
                version = "0.1",
            ),
            "tracing": crate.spec(
                version = "0.1.40",
            ),
            "tracing-subscriber": crate.spec(
                version = "0.3.18",
            ),
            "tracing-web": crate.spec(
                version = "0.1.3",
            ),
            "url": crate.spec(
                version = "2.5.0",
            ),
            "url-escape": crate.spec(
                version = "0.1.1",
            ),
            "web-sys": crate.spec(
                version = "0.3.77",
                features = ["Clipboard", "DomRectList", "DomRect", "DomRectReadOnly", "DomQuad", "File", "Url", "Blob", "BlobPropertyBag", "MediaQueryList", "Navigator", "Storage", "Window", "Performance", "HtmlCanvasElement", "CanvasRenderingContext2d", "Element", "HtmlElement", "CssStyleDeclaration"],
            ),
            "zip": crate.spec(
                version = "0.6.6",
                default_features = False,
                features = ["deflate", "time"],
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
        generator = "@cargo-bazel//:bin/cargo-bazel",
    )
