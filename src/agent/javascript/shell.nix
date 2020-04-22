{ pkgs ? import ../../../nix { inherit system; }
, system ? builtins.currentSystem
, agent-js ? import ./. { inherit pkgs; }
}:
pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  inputsFrom = [
    agent-js
  ];
}
