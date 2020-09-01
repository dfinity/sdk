{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
  # This should be a fs path to a checked-out agent-js git repo.
  # e.g. via niv at `nix-instantiate nix -A sources.agent-js-monorepo --eval`
, agent-js-monorepo-src
}:
let
  src = agent-js-monorepo-src;
  agentPackagePath = (src + "/packages/agent");
  # derivation that has all system dependencies required to build the npm monorepo:
  # * npm requires python3 to build with gyp
  # * on mac, npm may try to use fsevents
  monorepoSystemRequirements = pkgs.stdenv.mkDerivation {
    inherit src;
    name = "agent-js-monorepo-systemRequirements";
    propagatedNativeBuildInputs = [
      # Required by node-gyp
      pkgs.python3
    ];
    propagatedBuildInputs = [
      (
        pkgs.lib.optional pkgs.stdenv.isDarwin
          # Required by fsevents
          pkgs.darwin.apple_sdk.frameworks.CoreServices
      )
    ];
    installPhase = ''
      mkdir -p $out
    '';
  };
  monorepo = pkgs.napalm.buildPackage src {
    name = "agent-js-monorepo";
    propagatedBuildInputs = [
      monorepoSystemRequirements
    ];
    outputs = [
      "out"
      "lib"
      "agent"
      "bootstrap"
    ];
    installPhase = ''
      # $out: Everything!
      mkdir -p $out
      cp -R ./* $out/

      # $lib/node_modules: fetched npm dependencies
      mkdir -p $lib
      test -d node_modules && cp -R node_modules $lib || true

      # $agent: npm subpackage @dfinity/agent
      mkdir -p $agent
      cp -R node_modules $agent/
      cp -R ./packages/agent/* $agent/

      # $bootstrap: npm subpackage @dfinity/bootstrap
      mkdir -p $bootstrap
      cp -R node_modules $bootstrap/
      cp -R ./packages/bootstrap/* $bootstrap/
    '';
  };
in
monorepo
