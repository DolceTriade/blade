{
  lib,
  stdenv,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "cargo-bazel";
  version = "0.16.0";

  src = fetchFromGitHub {
    owner = "bazelbuild";
    repo = "rules_rust";
    rev = "09472f70de8ce1e8899239e677e12ceb64d05d90";
    hash = "sha256-t9vRRLRHzdIOGHZcCmYmFVHW4wXLXEaxDauJ0sNgumk=";
  };

  cargoHash = "sha256-1oxqe0Ce/E3qCYAqklA/ByrIjtpcRlync9pa+1mHxSE=";

  sourceRoot = "source/crate_universe";

  doCheck = false;

  RUSTFLAGS = "-Zlinker-features=-lld";

  buildNoDefaultFeatures = true;

  buildFeatures = ["cargo"];

  meta = {
    description = "A collection of tools which use Cargo to generate build targets for Bazel.";
    homepage = "https://github.com/bazelbuild/rules_rust";
    changelog = "https://github.com/bazelbuild/rules_rust/releases/tag/v${version}";
    license = with lib.licenses; [
      asl20
    ];
    mainProgram = "cargo-bazel";
  };
}
