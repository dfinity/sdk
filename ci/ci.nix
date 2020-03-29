{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, system ? builtins.currentSystem
, src ? {
    rev = pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);
    revCount = 0; # TODO: would be nice to get the `revCount` using Nix as well.
  }
, RustSec-advisory-db ? null

  # The version of the release. Will be set to the right value in ./release.nix.
, releaseVersion ? "latest"

  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? true

, pkgsPath ? ../nix
, pkgs ? import pkgsPath pkgsArgs
, pkgsArgs ? { inherit RustSec-advisory-db releaseVersion isMaster; }
}:
import ./mk-jobset.nix {
  inherit supportedSystems system pkgsPath pkgs pkgsArgs;
  inherit (src) rev;
  jobsetSpecificationPath = ../.;
  jobsArgs = { inherit RustSec-advisory-db releaseVersion isMaster src; };
}
