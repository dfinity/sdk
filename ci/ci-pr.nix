# This file is used to govern CI jobs for GitHub PRs

args@{supportedSystems ? [ "x86_64-linux" ], ...}:
import ./ci.nix (args // { inherit supportedSystems; })
