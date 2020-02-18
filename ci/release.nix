{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
, src ? null
}:
let
  pkgs = import ../nix {};
in
pkgs.ci ../release.nix
  {
    inherit supportedSystems scrubJobs src;
    rev = if src != null then src.rev else pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);
  }
