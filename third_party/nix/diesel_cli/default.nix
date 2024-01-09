{
  diesel-cli,
  rustPlatform,
  fetchCrate,
}: let
  diesel-cli' =
    diesel-cli.override
    {
      inherit rustPlatform;
      mysqlSupport = false;
    };
in
  diesel-cli'
  .overrideAttrs
  (self: super: rec {
    version = "2.1.1";
    src = with self;
      fetchCrate {
        inherit version;
        pname = "diesel_cli";
        hash = "sha256-fpvC9C30DJy5ih+sFTTMoiykUHqG6OzDhF9jvix1Ctg=";
      };

    cargoDeps = super.cargoDeps.overrideAttrs (_: {
      inherit src;
      outputHash = "sha256-nPmUCww8sOJwnG7+uIflLPgT87xPX0s7g0AcuDKhY2I";
    });
    doCheck = false;
  })
