{ pkgs ? (import ./. {}).pkgs }:
let dfx = pkgs.dfinity-sdk.shells.rust-workspace;
in
  pkgs.mkCiShell {
    name = "dfinity-sdk-env";
    inputsFrom = pkgs.stdenv.lib.attrValues pkgs.dfinity-sdk.shells;
    DFX_ASSETS = dfx.DFX_ASSETS;
  }
