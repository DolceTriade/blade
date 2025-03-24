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
    rev = "633e56db6dd2e5f9a0502ee068aa450baec51e7c";
    hash = "sha256-CuZlNmRR26YuRHOWVudf045RvQnNePMde0KrXLfLPX8=";
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
