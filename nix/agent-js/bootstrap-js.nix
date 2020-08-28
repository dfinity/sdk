{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
, sources ? import ../sources.nix { inherit system; }
, monorepo ? import ./agent-js-monorepo.nix { inherit system pkgs; }
}:
pkgs.stdenv.mkDerivation {
  name = "bootstrap-js";
  src = "${monorepo}/packages/bootstrap/";
  outputs = [
    "out"
    "lib"
  ];
  buildPhase = ''
  '';
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    test -d node_modules && cp -R node_modules $lib || true
  '';
}
