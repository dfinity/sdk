{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
, src ? null
}:
(import ../nix {}).ci ../release-jobset.nix { inherit supportedSystems scrubJobs src; }
