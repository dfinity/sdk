{ pkgs ? import ../. {} }:

let dfx = pkgs.dfinity-sdk.dfx; in

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-env";
  inputsFrom = [
    dfx
  ];
  DFX_ASSETS = dfx.DFX_ASSETS;
  shellHook = ''
    echo "{}" > dfinity.json

    # Clean up before we exit the shell
    trap "{ \
      rm dfinity.json
      exit 255; \
    }" EXIT
  '';
}
