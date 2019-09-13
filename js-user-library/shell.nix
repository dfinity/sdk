{ pkgs ? import ../. {} }:

let js-user-library = pkgs.dfinity-sdk.js-user-library; in

pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  inputsFrom = [
    js-user-library
  ];
}
