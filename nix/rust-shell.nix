{ pkgs ? (import ./. {}).pkgs }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  buildInputs = [pkgs.rls];
  nativeBuildInputs = [ pkgs.stdenv.cc ];
  inputsFrom = [ pkgs.dfinity-sdk.packages.shell ];
  shellHook = ''
    # Set CARGO_HOME to minimize interaction with any environment outside nix
    export CARGO_HOME=${pkgs.lib.dfinityRoot}/.cargo-home

    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
