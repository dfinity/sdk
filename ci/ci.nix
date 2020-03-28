{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, system ? builtins.currentSystem
, src ? null
, RustSec-advisory-db ? null

  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? true

  # The pkgs argument is needed such that ../default.nix can import and call this
  # file without causing a reevaluation of ./nix.
, pkgs ? null
}:
let
  # This functions creates a nixpkgs set for the given system.
  pkgsFor = system: import ../nix { inherit RustSec-advisory-db system isMaster; };

  # nixpkgs instantiated for the current system. Used primarily for its lib.
  pkgs' = if pkgs != null then pkgs else pkgsFor system;
  lib = pkgs'.lib;

  # An attribute set mapping every supported system to a nixpkgs instantiated
  # for that system. Special care is taken not to reevaluate nixpkgs for the
  # current system because we already did that in pkgs'.
  pkgsForSystem = lib.genAttrs supportedSystems (
    supportedSystem:
      if supportedSystem == system
      then pkgs'
      else pkgsFor supportedSystem
  );

  # An attribute set mapping every supported system to an attribute set of jobs
  # (as defined by ../default.nix) instantiated for that system.
  jobsForSystem = lib.genAttrs supportedSystems (
    system: import ../. {
      inherit src RustSec-advisory-db system isMaster;
      pkgs = pkgsForSystem."${system}";

      # We pass the final jobset such that jobs (like `publish.*`) can select
      # pre-evaluated jobs (like `dfx-release.x86_64-linux`) for a specific
      # system without requiring them to do any reevaluation.
      inherit jobset;
    }
  );

  # The final jobset defined as an attribute set as defined by ../default.nix
  # but with every derivation replaced with an attribute set mapping the system
  # that it should be build for to the derivation instantiated for that
  # system. For example:
  #
  # jobset = {
  #   ...
  #   dfx.build.x86_64-linux = derivation { system = "x86_64-linux"; ... };
  #   dfx.build.x86_64-darwin = derivation { system = "x86_64-darwin"; ... };
  #   publish.dfx.x86_64-linux = derivation { system = "x86_64-linux"; ... };
  #   ...
  # }
  #
  # See the following for the current list of jobs:
  # https://hydra.dfinity.systems/jobset/dfinity-ci-build/sdk#tabs-jobs
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

  supportedPlatforms =
    map (system: lib.systems.elaborate { inherit system; }) supportedSystems;

  # The `all-jobs` is an aggregate job consisting of all other jobs.
  # This job should build successfully to allow PRs to merge.
  all-jobs = pkgs'.releaseTools.aggregate {
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
      rev = if src != null then src.rev else lib.commitIdFromGitRepo (lib.gitDir ../.);

      allowSubstitutes = false;
    }
  );
}
