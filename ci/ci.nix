{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
}:
let pkgs = import ../nix {};
in
pkgs.ci ../jobset.nix
  { inherit supportedSystems scrubJobs; isMaster = true;
    rev = pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);
  }
