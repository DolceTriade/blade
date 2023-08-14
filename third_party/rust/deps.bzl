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
                        "AR": "external/cctools/bin/ar",
                    },
                    build_script_tools = [
                        "@cctools//:bin/ar",
                    ],
                ),
            ],
        },
        packages = {
            "actix-files": crate.spec(
                version = "0.5",
            ),
            "actix-web": crate.spec(
                version = "4",
                features = ["macros"],
            ),
            "broadcaster": crate.spec(
                version = "1",
            ),
            "console_log": crate.spec(
                version = "1",
            ),
            "console_error_panic_hook": crate.spec(
                version = "0.1",
            ),
            "futures": crate.spec(
                version = "0.3",
            ),
            "cfg-if": crate.spec(
                version = "1",
            ),
            "lazy_static": crate.spec(
                version = "1",
            ),
            "leptos": crate.spec(
                version = "0.4.8",
            ),
            "leptos_actix": crate.spec(
                version = "0.4.8",
            ),
            "leptos_meta": crate.spec(
                version = "0.4.8",
            ),
            "leptos_router": crate.spec(
                version = "0.4.8",
            ),
            "log": crate.spec(
                version = "0.4",
            ),
            "serde": crate.spec(
                version = "1",
                features = ["derive"],
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
    )
