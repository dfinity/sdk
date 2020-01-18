{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
, src ? null
}:
let
  pkgs = import ../nix {};
in
pkgs.ci ../release-jobset.nix
  {
    inherit supportedSystems scrubJobs src;
    rev = pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);
  }
