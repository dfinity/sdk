{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
, sources ? import ../sources.nix { inherit system; }
}:
let
  src = sources.agent-js-monorepo;
in

# This does not work. napalm doesn't like how `npm install` triggers lerna bootstrap.
pkgs.napalm.buildPackage src {
  name = "agent-js-monorepo";
  outputs = [ "out" "lib" ];
  npmCommands = [
    "npm install"
  ];
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    test -d node_modules && cp -R node_modules $lib || true
  '';
}
