{ system ? builtins.currentSystem
, isMaster ? true
, pkgs ? import ./nix { inherit system isMaster labels; }
, labels ? {}
, use_ic_ref ? false
, dfx
}:
let
  inherit (pkgs) lib;

  args = {
    inherit pkgs dfx system use_ic_ref;

    utils = lib.gitOnlySource ./utils;
    assets = lib.gitOnlySource ./assets;
  };
in
{ dfx = import ./tests-dfx args;
  replica = import ./tests-replica args;
}
