{ pkgs ? import ../. {} }:

let dfx = pkgs.dfinity-sdk.dfx; in

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-env";
  inputsFrom = [
    dfx
  ];
  shellHook = ''
    export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
  '';
  DFX_ASSETS = dfx.DFX_ASSETS;
}
