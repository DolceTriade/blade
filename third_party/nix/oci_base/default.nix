let
    pkgs = import <nixpkgs>;
    packages = with pkgs; [
        postgresql.lib
        sqlite.out
    ];
    closure = with pkgs; builtins.toString (lib.strings.splitString "\n" (builtins.readFile "${closureInfo {rootPaths = packages;}}/store-paths"));
in
    pkgs.buildEnv {
    name = "closure";
    paths = [];
    buildInputs = packages;
    postBuild = "tar -cf $out/closure.tar ${closure}";
  }