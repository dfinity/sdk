{ pkgs ? import ./nix {}, rust-package ? import ./dfx { inherit pkgs; } }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-rust-env";
  buildInputs = [ pkgs.rls ];
  inputsFrom = [ rust-package.shell ];
  shellHook = ''
    # Set CARGO_HOME to minimize interaction with any environment outside nix
    export CARGO_HOME=${if pkgs.lib.isHydra then "." else toString ../.}/.cargo-home

    # Set environment variable for debug version.
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
}
