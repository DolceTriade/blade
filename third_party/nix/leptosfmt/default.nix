{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:
rustPlatform.buildRustPackage rec {
  pname = "leptosfmt";
  version = "8b4194ba33eee417ababdd15498940014fd6d237";

  src = fetchFromGitHub {
    owner = "bram209";
    repo = "leptosfmt";
    rev = "8b4194ba33eee417ababdd15498940014fd6d237";
    hash = "sha256-F06Ag99rCn3qZywdxyP7ULOgyhbSzWNe+drBDZJWVxo=";
    fetchSubmodules = true;
  };

  cargoHash = "sha256-Q2S34G4pcYv8sC/gE2+Q5ovVrrLBNV/Ltt0Ex/vuJTU=";

  meta = with lib; {
    description = "Formatter for the leptos view! macro";
    mainProgram = "leptosfmt";
    homepage = "https://github.com/bram209/leptosfmt";
    changelog = "https://github.com/bram209/leptosfmt/blob/${src.rev}/CHANGELOG.md";
    license = with licenses; [
      asl20
      mit
    ];
    maintainers = with maintainers; [figsoda];
  };
}
