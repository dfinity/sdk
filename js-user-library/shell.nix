{ pkgs ? (import ../. {}).pkgs }:

let js-user-library = pkgs.dfinity-sdk.packages.js-user-library; in

pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  inputsFrom = [
    js-user-library
  ];
}
