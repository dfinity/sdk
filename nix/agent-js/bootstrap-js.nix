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
    npm run build --if-present
  '';
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    test -d node_modules && cp -R node_modules $lib || true
  '';
}
