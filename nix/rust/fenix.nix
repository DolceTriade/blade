let
  lock = builtins.fromJSON (builtins.readFile ../../flake.lock);
  fenixSrc = lock.nodes.fenix.locked;
  fenix = assert fenixSrc.type == "github";
    fetchTarball {
      url = "https://github.com/${fenixSrc.owner}/${fenixSrc.repo}/archive/${fenixSrc.rev}.tar.gz";
      sha256 = fenixSrc.narHash;
    };
in
  import fenix
