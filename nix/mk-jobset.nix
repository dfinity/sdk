pkgs:
# This function returns a Hydra jobset like for example:
#
#   {
#     ...
#     dfx.build.x86_64-linux = derivation { system = "x86_64-linux"; ... };
#     dfx.build.x86_64-darwin = derivation { system = "x86_64-darwin"; ... };
#     publish.dfx.x86_64-linux = derivation { system = "x86_64-linux"; ... };
#     ...
#   }
#
# by applying it to a jobset specification like:
#
#   {
#     ...
#     dfx.build = derivation { ... };
#     publish.dfx = lib.linuxOnly ( derivation { ... } );
#     ...
#   }
#
# So it transforms every top-level derivation in the specification to a set of
# jobs for every supported system (filtered by what the derivation supports).
#
# This function should be used in `common`, `dfinity`, `sdk` and `infra` to
# define their Hydra jobsets.
#
# TODO: This function is independent of this repo and should be moved to
# `common` eventually. See: https://dfinity.atlassian.net/browse/INF-1151
{
  # A list of strings specifying for which systems to build jobs.
  #
  # Note that derivations can filter this list be setting the `meta.platforms`
  # attribute or using the `lib.linuxOnly` function which does it for them.
  supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]

  # The system used to evaluate the nixpkgs set used in the implementation of
  # this function (mainly to get access to its `lib`) and to evaluate the jobset
  # specification which determines the final Hydra jobset.
  #
  # This should only be overridden in debugging scenarios.
, system ? pkgs.system

  # The git revision used in the `all-jobs` job.
, rev

  # Path to the jobset specification.
  #
  # It's recommended to standardize this to `../.`.
  #
  # Note that the jobset specification will be called with the following
  # arguments:
  #
  #   jobsetSpecificationArgs // {
  #     inherit system;
  #     pkgs = pkgs.pkgsForSystem."${system}";
  #
  #     # We pass the final jobset such that jobs (like `publish.*` in the `sdk`
  #     # repo) can select pre-evaluated jobs (like `dfx-release.x86_64-linux`)
  #     # for a specific system without requiring them to do any reevaluation.
  #     inherit jobset;
  #   }
  #
  # for every supported `system`.
, jobsetSpecificationPath

  # Extra arguments to the `jobsetSpecificationPath` function.
, jobsetSpecificationArgs ? {}
}:
let
  lib = pkgs.lib;

  # An attribute set mapping every supported system to an attribute set of jobs
  # (as defined by `jobsetSpecificationPath`) evaluated for that system.
  jobsForSystem = lib.genAttrs supportedSystems (
    system: import jobsetSpecificationPath (
      jobsetSpecificationArgs // {
        inherit system;
        pkgs = pkgs.pkgsForSystem."${system}";

        # We pass the final jobset such that jobs (like `publish.*` in the `sdk`
        # repo) can select pre-evaluated jobs (like `dfx-release.x86_64-linux`)
        # for a specific system without requiring them to do any re-evaluation.
        inherit jobset;
      }
    )
  );

  # The final Hydra jobset defined as an attribute set as specified by the
  # jobset specification but with every derivation replaced with an attribute
  # set mapping the system that it should be build for to the derivation
  # instantiated for that system. For example:
  #
  # jobset = {
  #   ...
  #   dfx.build.x86_64-linux = derivation { system = "x86_64-linux"; ... };
  #   dfx.build.x86_64-darwin = derivation { system = "x86_64-darwin"; ... };
  #   publish.dfx.x86_64-linux = derivation { system = "x86_64-linux"; ... };
  #   ...
  # }
  jobset = lib.mapAttrsRecursiveCond
    (as: !lib.isDerivation as) createJobsForDrv jobsForSystem."${system}";
  createJobsForDrv = path: drv:
    let
      # All supported systems filtered by what's specified by the derivation's
      # `meta.platforms` attribute.
      supportedSystemsByDrv =
        if drv ? meta.platforms
        then supportedMatches drv.meta.platforms
        else supportedSystems;
    in
      lib.genAttrs supportedSystemsByDrv (
        system: lib.getAttrFromPath path jobsForSystem."${system}"
      );

  # (Copied from <nixpkgs/pkgs/top-level/release-lib.nix> then modified).
  #
  # Given a list of 'meta.platforms'-style patterns, return the sublist of
  # `supportedSystems` containing systems that matches at least one of the given
  # patterns.
  supportedMatches = metaPatterns:
    let
      anyMatch = platform:
        lib.any (lib.meta.platformMatch platform) metaPatterns;
      matchingPlatforms = lib.filter anyMatch supportedPlatforms;
    in
      map ({ system, ... }: system) matchingPlatforms;

  # The supported systems converted into platforms such that they can be matched
  # with the derivation's `meta.platforms`.
  supportedPlatforms =
    map (system: lib.systems.elaborate { inherit system; }) supportedSystems;

  # The `all-jobs` is an aggregate job consisting of all other jobs.
  # This job should build successfully to allow PRs to merge.
  all-jobs = pkgs.releaseTools.aggregate {
    name = "all-jobs";
    constituents = lib.collect lib.isDerivation jobset;
  };
in
jobset // {
  all-jobs = all-jobs.overrideAttrs (
    _oldAttrs: {
      # We need to make sure the `all-jobs` job is always rebuilt
      # for every git commit. So we set a `rev` attribute on the
      # derivation to the current git HEAD revision. Every time the git
      # HEAD changes the `rev` attribute changes. This causes the
      # derivation to be different causing it to be rebuild.
      inherit rev;

      allowSubstitutes = false;
    }
  );
}
