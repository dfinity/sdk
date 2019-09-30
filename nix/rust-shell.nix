{ pkgs ? (import ./. {}).pkgs }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  inputsFrom = [ pkgs.dfinity-sdk.packages.rust-workspace ];
  shellHook = ''
    # Set CARGO_HOME to minimize interaction with any environment outside nix
    export CARGO_HOME=${pkgs.lib.dfinityRoot}/.cargo-home

    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
