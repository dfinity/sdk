{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, use_ic_ref ? false
}:
let
  inherit (pkgs) lib;

  isBatsTest = fileName: type: lib.hasSuffix ".bash" fileName && type == "regular";

  here = ./.;

  mkBatsTest = fileName:
    let
      name = lib.removeSuffix ".bash" fileName;
    in
      lib.nameValuePair name (
        pkgs.runCommandNoCC "e2e-test-${name}${lib.optionalString use_ic_ref "-use_ic_ref"}" {
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
          utils = lib.gitOnlySource ./utils;
          assets = lib.gitOnlySource ./assets;
          test = here + "/${fileName}";
        } ''
          export HOME=$(pwd)

          ln -s $utils utils
          ln -s $assets assets
          ln -s $test test

          # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
          # less resources than a dev's computer, so e2e might take longer.
          timeout --preserve-status 3600 bats test | tee $out
        ''
      );
in
builtins.listToAttrs
  (
    builtins.map mkBatsTest
      (
        lib.attrNames
          (
            lib.filterAttrs isBatsTest
              (builtins.readDir here)
          )
      )
  )
