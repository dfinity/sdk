# Returns a set of jobs that Hydra will build on the `master` branch.
#
# This file returns a job for each supported platform for each job
# defined in `../jobset.nix`. For example, while `../jobset.nix`
# defines the `dfinity-sdk.dfx` job, this file returns:
#
# {
#   dfinity-sdk.dfx.x86_64-linux = ...;
#   dfinity-sdk.dfx.x86_64-darwin = ...;
# }
{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
}:

let
  pkgs = (import ../. {}).pkgs;
  sources = import ../nix/sources.nix;

  release-lib = import ((sources.nixpkgs) + "/pkgs/top-level/release-lib.nix") {
    inherit supportedSystems scrubJobs;
    packageSet = import ../jobset.nix;
    # These arguments are passed into the jobset function.
    nixpkgsArgs = {
      config = {
        inHydra = true;
      };
    };
  };

  lib = pkgs.lib;

  jobs = release-lib.mapTestOn (lib.mapAttrsRecursiveCond (as: !lib.isDerivation as) (_path: drv:
    release-lib.supportedMatches (drv.meta.platforms or supportedSystems)
  ) release-lib.pkgs);

in jobs // rec {

  all-jobs =
    (pkgs.releaseTools.aggregate {
      name = "all-jobs";
      constituents = lib.collect (drv: lib.isDerivation drv) jobs;
    }).overrideAttrs (_oldAttrs: {
      allowSubstitutes = false;

      # We need to make sure the `all-systems-go` job is always rebuild
      # for every git commit. So we set a `rev` attribute on the
      # derivation to the current git HEAD revision. Every time the git
      # HEAD changes the `rev` attribute changes. This causes the
      # derivation to be different causing it to be rebuild.
      rev = pkgs.lib.commitIdFromGitRepo ../.git;
    });
}
