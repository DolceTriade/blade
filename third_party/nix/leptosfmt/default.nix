{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "leptosfmt";
  version = "0.1.18";

  src = fetchFromGitHub {
    owner = "bram209";
    repo = pname;
    rev = version;
    hash = "sha256-bNfTZgcru7PJR/9AcaOmW0E8QwdiXcuP7MWXcDPXGso=";
  };

  cargoHash = "sha256-NQYIq9Wc2mtUGeS3Iv2e0nfQkvcX6hOxZ6FHVcHD5cs=";

  meta = with lib; {
    description = "A formatter for the leptos view! macro.";
    homepage = "https://github.com/bram209/leptosfmt";
    license = licenses.asl20;
    maintainers = [];
  };
}
