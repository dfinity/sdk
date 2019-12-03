{ pkgs ? (import ./. {}).pkgs, shell }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  buildInputs = [pkgs.rls];
  nativeBuildInputs = [ pkgs.stdenv.cc ];
  inputsFrom = [ shell ];
  shellHook = ''
    # Set CARGO_HOME to minimize interaction with any environment outside nix
    export CARGO_HOME=${if pkgs.lib.isHydra then "." else toString ../.}/.cargo-home

    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
