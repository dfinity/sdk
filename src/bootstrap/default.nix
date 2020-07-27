{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, agent-js ? import ../agent/javascript { inherit pkgs; }
}:
pkgs.napalm.buildPackage (pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../../. ./.)) {
  root = ./.;
  name = "bootstrap-js";
  buildInputs = [ agent-js ];

  outputs = [ "out" "lib" ];

  propagatedNativeBuildInputs = [
    # Required by node-gyp
    pkgs.python3
  ];
  propagatedBuildInputs = pkgs.lib.optional pkgs.stdenv.isDarwin
    # Required by fsevents
    pkgs.darwin.apple_sdk.frameworks.CoreServices;

  # ci script now does everything CI should do. Bundle is needed because it's the output
  # of the nix derivation.
  npmCommands = [
    "npm install"
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
    "npm run bundle"
  ];

  installPhase = ''
    mkdir -p $out

    cp -R dist/* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    cp -R node_modules $lib
  '';
}
