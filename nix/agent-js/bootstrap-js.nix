{ pkgs ? import ../. { inherit system; }, system ? builtins.currentSystem }:
pkgs.stdenv.mkDerivation {
  name = "agent-js-monorepo-package-bootstrap";
  src = "${pkgs.agent-js-monorepo}";
  buildInputs = [
    pkgs.agent-js-monorepo
    pkgs.nodejs
  ];
  outputs = [
    "out"
    "lib"
    "dist"
  ];
  configurePhase = ''
    export HOME=$(mktemp -d)
  '';
  unpackPhase = ''
    mkdir bootstrap-bundle
    cp -R ${pkgs.agent-js-monorepo}/* bootstrap-bundle/
  '';
  installPhase = ''
    # $out: everything
    mkdir -p $out
    cp -R ${pkgs.agent-js-monorepo.bootstrap}/* $out/

    # $lib/node_modules: node_modules dir that must be resolvable by npm
    #   for future build steps to work (e.g. at ../../node_modules)
    mkdir -p $lib
    if test -d node_modules; then
      cp -R node_modules $lib;
    fi
    
    # $dist: Store src files as outputed from typescript compiler
    mkdir -p $dist
    dist_src="bootstrap-bundle/packages/bootstrap/dist"
    if test -d "$dist_src"; then
      cp -R $dist_src/* $dist/
    fi
  '';
}
