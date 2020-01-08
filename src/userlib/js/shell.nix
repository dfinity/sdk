{ pkgs ? import ../../../nix { inherit system; }
, system ? builtins.currentSystem
}:

let js-user-library = pkgs.dfinity-sdk.packages.userlib.js; in

pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  inputsFrom = [
    js-user-library
  ];
}
