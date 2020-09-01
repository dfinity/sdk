{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
}:
pkgs.stdenv.mkDerivation {
  name = "agent-js-monorepo-package-agent";
  src = "${pkgs.agent-js-monorepo}/packages/agent";
  buildInputs = [ pkgs.nodejs ];
  outputs = [
    "out"
    "lib"
  ];
  buildPhase = ''
    # Don't run `npm run build` here, which will call `tsc -b`.
    # `tsc -b` will use typescript project references to build things,
    # which may try to read from other packages, which will fail due to writing in an external nix store.
    # We expect pkgs.agent-js-monorepo to have already taken care of the `npm install` part of fetching deps.
  '';
  installPhase = ''
    # $out: everything
    mkdir -p $out
    cp -R ./* $out/

    # $lib/node_modules: node_modules dir that must be resolvable by npm
    #   for future build steps to work (e.g. at ../../node_modules)
    mkdir -p $lib
    if test -d node_modules; then
      cp -R node_modules $lib;
    fi
  '';
}
