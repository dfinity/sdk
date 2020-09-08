{ pkgs ? import ../. { inherit system; }, system ? builtins.currentSystem }:
pkgs.stdenv.mkDerivation {
  name = "agent-js-monorepo-package-agent";
  src = "${pkgs.agent-js-monorepo}";
  outputs = [
    "out"
    "lib"
  ];
  buildInputs = [
    pkgs.nodejs
    pkgs.agent-js-monorepo
  ];
  configurePhase = ''
    export HOME=$(mktemp -d)
  '';
  installPhase = ''
    # $out: everything
    mkdir -p $out
    cp -R ${pkgs.agent-js-monorepo.agent}/* $out/

    # $lib/node_modules: node_modules dir that must be resolvable by npm
    #   for future build steps to work (e.g. at ../../node_modules)
    mkdir -p $lib
    agent_node_modules="${pkgs.agent-js-monorepo}/packages/agent/node_modules"
    if test -d "$agent_node_modules"; then
      cp -R "$agent_node_modules" $lib;
    fi
  '';
}
