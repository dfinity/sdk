{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, agent-js ? import ../../src/agent/javascript { inherit pkgs; }
}:
pkgs.napalm.buildPackage (pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ./.)) {
  root = ./.;
  name = "node-e2e-tests";
  buildInputs = [ dfx.standalone agent-js ];

  npmCommands = [
    "npm install"

    # Monkey-patch the agent source into our install dir. napalm is unable
    # to include dependencies from package-locks in places other than the
    # build root.
    (
      pkgs.writeScript "include-agent.sh" ''
        #!${pkgs.stdenv.shell}
        set -eo pipefail

        agent_node_modules="node_modules/@dfinity/agent"
        mkdir -p $agent_node_modules

        tar xvzf ${agent-js.out}/dfinity-*.tgz --strip-component 1 --directory $agent_node_modules/
        cp -R ${agent-js.lib}/node_modules .
      ''
    )
    "npm run ci"
  ];

  installPhase = ''
    echo Done.
    touch $out
  '';
}
