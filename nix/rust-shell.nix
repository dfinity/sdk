{ pkgs ? (import ./. {}).pkgs }:
let rust-workspace = pkgs.dfinity-sdk.packages.rust-workspace;
    rustc = pkgs.rustPackages.rustc;
in
  pkgs.mkCiShell {
    name = "dfinity-sdk-rust-env";
    inputsFrom = [ rust-workspace ];
    shellHook = ''
      # Make sure our specified rustc is in front of the PATH so cargo will use.
      export PATH="${rustc}/bin''${PATH:+:}$PATH"

      # Set CARGO_HOME to minimize interaction with any environment outside nix
      export CARGO_HOME=${pkgs.lib.dfinityRoot}/.cargo-home

      # Set environment variable for debug version.
      export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
    '';
    DFX_ASSETS = pkgs.dfinity-sdk.packages.rust-workspace.DFX_ASSETS;
  }
