{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, system ? builtins.currentSystem
, src ? builtins.fetchGit ../.
, RustSec-advisory-db ? pkgs.sources.advisory-db

  # The version of the release. Will be set to the right value in ./release.nix.
, releaseVersion ? "latest"

, pkgs ? import ../nix { inherit system; }
}:
pkgs.lib.mk-jobset {
  inherit supportedSystems;
  inherit (src) rev;
  mkJobsetSpec = { system, pkgs, jobset }: import ../. {
    inherit system pkgs jobset RustSec-advisory-db releaseVersion src;
  };
}
