{ pkgs ? import ../../../nix { inherit system; }
, system ? builtins.currentSystem
, userlib-js ? import ./. { inherit pkgs; }
}:
pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  inputsFrom = [
    userlib-js
  ];
}
