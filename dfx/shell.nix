{ pkgs ? import ../. {} }:

let dfx = pkgs.dfinity-sdk.dfx; in

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-env";
  inputsFrom = [
    dfx
  ];
  DFX_ASSETS = dfx.DFX_ASSETS;
}
