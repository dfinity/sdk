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
    archive = lib.gitOnlySource ./archive;
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
      nodejs
      ps
      python3
      procps
      which
      dfx.build
    ] ++ lib.optional use_ic_ref ic-ref
    ++ lib.optional stdenv.isLinux [ glibc.bin patchelf ];
    BATSLIB = pkgs.sources.bats-support;
    USE_IC_REF = use_ic_ref;
    assets = args.assets;
    utils = args.utils;
    archive = args.archive;
  } ''
    touch $out
  '';


  recurseForDerivations = true;
}
