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
{
  dfx = import ./tests-dfx args;
  replica = import ./tests-replica args;

  shell = pkgs.runCommandNoCC "e2e-test-shell${lib.optionalString use_ic_ref "-use_ic_ref"}" {
    nativeBuildInputs = with pkgs; [
      bats
      diffutils
      curl
      findutils
      gnugrep
      gnutar
      gzip
      jq
      mitmproxy
      netcat
      ps
      python3
      procps
      which
      dfx.standalone
    ] ++ lib.optional use_ic_ref ic-ref;
    BATSLIB = pkgs.sources.bats-support;
    USE_IC_REF = use_ic_ref;
    assets = args.assets;
    utils = args.utils;
  } ''
    touch $out
  '';


  recurseForDerivations = true;
}
