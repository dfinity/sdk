{ pkgs ? import ../. {} }:

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-demo";
  buildInputs = [
    pkgs.dfinity-sdk.dfx
  ];
}
