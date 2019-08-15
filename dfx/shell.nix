{ pkgs ? import ../. {} }:

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-env";
  inputsFrom = [
    pkgs.dfinity-sdk.dfx
  ];
}
