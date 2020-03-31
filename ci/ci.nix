{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, system ? builtins.currentSystem
, src ? builtins.fetchGit ../.
, RustSec-advisory-db ? null

  # The version of the release. Will be set to the right value in ./release.nix.
, releaseVersion ? "latest"

  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? true

, pkgs ? import ../nix { inherit system isMaster RustSec-advisory-db; }
}:
pkgs.lib.mk-jobset {
  inherit supportedSystems;
  inherit (src) rev;
  jobsetSpecificationPath = ../.;
  jobsetSpecificationArgs = { inherit RustSec-advisory-db releaseVersion isMaster src; };
}
