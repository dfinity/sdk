{ pkgs ? (import ./. {}).pkgs }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  inputsFrom = [ pkgs.dfinity-sdk.packages.rust-workspace ];
  shellHook = ''
    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
