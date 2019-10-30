{ pkgs ? (import ./. {}).pkgs }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  inputsFrom = [

    (pkgs.dfinity-sdk.packages.rust-workspace-debug.overrideAttrs (_oldAttrs: {
      # _oldAttrs.configurePhase refers to the dfinity-application-and-others-deps
      # derivation which is the build of all 3rd-party Rust dependencies. Since in this
      # nix-shell we use cargo locally to build all dependencies we don't need to depend
      # on this derivation saving a lot of time downloading/building.
      configurePhase = "";
    })) ];

  shellHook = ''
    # Set CARGO_HOME to minimize interaction with any environment outside nix
    export CARGO_HOME=${pkgs.lib.dfinityRoot}/.cargo-home

    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
