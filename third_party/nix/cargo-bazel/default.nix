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
    rev = "0.59.2";
    hash = "sha256-N+O2/HxQ/zlscwA029wA9sl0MaCYqbv5ULPVCugbVL0=";
  };

  cargoHash = "sha256-Bu3mLlxC6i4wXSs4KNGEc15Qdr92dJfYbmY50NyM3nk=";

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
