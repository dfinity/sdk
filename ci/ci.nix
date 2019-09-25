{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
}:
(import ../nix {}).ci ../jobset.nix { inherit supportedSystems scrubJobs; }
