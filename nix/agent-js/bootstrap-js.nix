{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
}:
pkgs.stdenv.mkDerivation {
  name = "bootstrap-js";
  src = "${pkgs.agent-js-monorepo}/packages/bootstrap/";
  buildInputs = [ pkgs.nodejs ];
  outputs = [
    "out"
    "lib"
  ];
  buildPhase = ''
    # Don't run `npm run build` here, which will call `tsc -b`.
    # `tsc -b` will use typescrpit project references to build things,
    # which may try to read from other packages. But nix will error on read from those dirs.
    # Let all building happen on the monorepo package.
  '';
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    test -d node_modules && cp -R node_modules $lib || true
  '';
}
