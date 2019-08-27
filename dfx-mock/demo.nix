{ pkgs ? import ../. {} }:

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-mock-demo";
  buildInputs = [
    pkgs.dfinity-sdk.dfx-mock
  ];
}

