{ pkgs ? import ../. {} }:

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-mock-env";
  inputsFrom = [
    pkgs.dfinity-sdk.dfx-mock
  ];
}
