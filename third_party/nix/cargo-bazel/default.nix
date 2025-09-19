{
  lib,
  stdenv,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "cargo-bazel";
  version = "0.17.0";

  src = fetchFromGitHub {
    owner = "bazelbuild";
    repo = "rules_rust";
    rev = "0.65.0";
    hash = "sha256-Q1FenqIxp2Qy++cSu5RFdGzrQmqbCfKRZB1VR97v77Q=";
  };

  cargoHash = "sha256-beOFmmeAK2cNANxacv4GfJqEptvqD1/CNJ+Mmunb7/Y=";

  sourceRoot = "source/crate_universe";

  doCheck = false;

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
