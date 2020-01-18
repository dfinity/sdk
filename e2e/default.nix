{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
}:
let
  e2e = lib.noNixFiles (lib.gitOnlySource ../. "e2e");
  lib = pkgs.lib;
  sources = pkgs.sources;
in
pkgs.runCommandNoCC "e2e-tests" {
  __darwinAllowLocalNetworking = true;
  buildInputs = with pkgs; [ bats coreutils curl dfinity-sdk.packages.rust-workspace-debug nodejs stdenv.cc ps python3 netcat which ];
} ''
  # We want $HOME/.cache to be in a new temporary directory.
  export HOME=$(mktemp -d -t dfx-e2e-home-XXXX)
  # We use BATSLIB in our scripts to find the root of the BATSLIB repo.
  export BATSLIB="${sources.bats-support}"

  # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
  # less resources than a dev's computer, so e2e might take longer.
  timeout --preserve-status 600 bats --recursive ${e2e}/* | tee $out
''
