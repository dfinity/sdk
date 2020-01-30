{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
}:
let
  e2e = lib.noNixFiles (lib.gitOnlySource ../. "e2e");
  lib = pkgs.lib;
  sources = pkgs.sources;

  inputs = with pkgs; [
    bats bash coreutils curl findutils gnugrep gnutar gzip netcat ps python3 which
    dfinity-sdk.packages.rust-workspace-standalone
  ];
in
builtins.derivation {
  name = "e2e-tests";
  inherit system;
  PATH = pkgs.lib.makeSearchPath "bin" inputs;
  BATSLIB = sources.bats-support;
  builder = [ (pkgs.writeScript "builder.sh" ''
    #!${pkgs.stdenv.shell} 
    set -eo pipefail

    # We want $HOME/.cache to be in a new temporary directory.
    export HOME=$(mktemp -d -t dfx-e2e-home-XXXX)

    # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
    # less resources than a dev's computer, so e2e might take longer.
    timeout --preserve-status 600 bats --recursive ${e2e}/* | tee $out
  '') ];
}
