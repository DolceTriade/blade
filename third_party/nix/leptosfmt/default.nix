{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "leptosfmt";
  version = "7fec90b22e1dac9a4649aefbccee2bffc449fbb3";

  src = fetchFromGitHub {
    owner = "bram209";
    repo = pname;
    rev = version;
    hash = "sha256-kozg49iWJbB5RZomVu6aLStv+YTjcsGD3sUO4tjS5r4=";
  };

  cargoHash = "sha256-hAuRstAlHONmwiOwk6QqV2rugqFwT0QGY/sfRZohsmo=";

  meta = with lib; {
    description = "A formatter for the leptos view! macro.";
    homepage = "https://github.com/bram209/leptosfmt";
    license = licenses.asl20;
    maintainers = [];
  };
}
