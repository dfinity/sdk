{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, use_ic_ref ? false
, archive
, assets
, utils
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
            nodejs
            ps
            python3
            procps
            which
            dfx.build
          ] ++ lib.optional stdenv.isLinux [ glibc.bin patchelf ];
          BATSLIB = pkgs.sources.bats-support;
          USE_IC_REF = use_ic_ref;
          archive = archive;
          assets = assets;
          utils = utils;
          test = here + "/${fileName}";
        } ''
          export HOME=$(pwd)

          ln -s $utils utils
          ln -s $assets assets
          ln -s $archive archive
          mkdir test
          ln -s $test test/test.bash

          # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
          # less resources than a dev's computer, so e2e might take longer.
          timeout --preserve-status 3600 bats test/test.bash | tee $out
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
  ) // { recurseForDerivations = true; }
