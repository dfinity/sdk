# This file is used to govern CI jobs for GitHub PRs

args@{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ], ... }:
import ./ci.nix (args // { inherit supportedSystems; isMaster = false; })
