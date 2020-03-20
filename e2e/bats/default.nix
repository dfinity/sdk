{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, use_ic_ref ? true
}:
let
  e2e = lib.noNixFiles (lib.gitOnlySource ../../. ./.);
  lib = pkgs.lib;
  sources = pkgs.sources;

  inputs = with pkgs; [
    bats
    bash
    coreutils
    diffutils
    curl
    findutils
    gnugrep
    gnutar
    gzip
    netcat
    ps
    python3
    procps
    which
    dfx.standalone
    ic-ref
  ];
in

builtins.derivation {
  name = "e2e-tests";
  system = pkgs.stdenv.system;
  PATH = pkgs.lib.makeSearchPath "bin" inputs;
  BATSLIB = sources.bats-support;
  builder =
    pkgs.writeScript "builder.sh" ''
      #!${pkgs.stdenv.shell}
      set -eo pipefail

      # We want $HOME/.cache to be in a new temporary directory.
      export HOME=$(mktemp -d -t dfx-e2e-home-XXXX)

      export USE_IC_REF=${if use_ic_ref then "true" else "false"}

      # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
      # less resources than a dev's computer, so e2e might take longer.
      timeout --preserve-status 600 bats --recursive ${e2e}/* | tee $out
    '';
} // { meta = {}; }
