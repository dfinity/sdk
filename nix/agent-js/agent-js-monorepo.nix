{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
  # This should be via sourcesnix for the git monorepo
, agent-js-monorepo-src
}:
let
  src = agent-js-monorepo-src;
  agentPackagePath = (src + "/packages/agent");
  monorepo = pkgs.napalm.buildPackage src {
    name = "agent-js-monorepo";
    buildInputs = [
      pkgs.python3
    ];
    propagatedNativeBuildInputs = [
      # Required by node-gyp
      pkgs.python3
    ];
    propagatedBuildInputs = pkgs.lib.optional pkgs.stdenv.isDarwin
      # Required by fsevents
      pkgs.darwin.apple_sdk.frameworks.CoreServices;
    outputs = [
      "out"
      "agent"
      "bootstrap"
      "lib"
    ];
    # HUSKY_DEBUG = "1";
    # HUSKY_SKIP_INSTALL = "1";
    npmCommands = [
      "npm install"
    ];
    installPhase = ''
      mkdir -p $out

      cp -R ./* $out/

      # Copy node_modules to be reused elsewhere.
      mkdir -p $lib
      test -d node_modules && cp -R node_modules $lib || true

      mkdir -p $agent
      cp -R node_modules $agent/
      cp -R ./packages/agent/* $agent/

      mkdir -p $bootstrap
      cp -R node_modules $bootstrap/
      cp -R ./packages/bootstrap/* $bootstrap/
    '';
  };
in
monorepo
