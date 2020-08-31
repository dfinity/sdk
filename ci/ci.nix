{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, system ? builtins.currentSystem
, src ? builtins.fetchGit ../.
, RustSec-advisory-db ? pkgs.sources.advisory-db

, isMaster ? true
, labels ? {}

  # The version of the release. Will be set to the right value in ./release.nix.
, releaseVersion ? "latest"

, pkgs ? import ../nix { inherit system isMaster labels; }
}:
let
  jobset =
    pkgs.lib.mk-jobset {
      inherit supportedSystems;
      inherit (src) rev;
      mkJobsetSpec = { system, pkgs, ... }: import ../. {
        inherit system pkgs RustSec-advisory-db releaseVersion src;
      };
    };

  publish = import ./publish.nix {
    inherit pkgs releaseVersion;
    inherit (jobset) install;
    dfx = jobset.dfx.standalone;
  };
in
jobset // { inherit publish; }
