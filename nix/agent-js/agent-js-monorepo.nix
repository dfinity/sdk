{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
  # This should be a fs path to a checked-out agent-js git repo.
  # e.g. via niv at `nix-instantiate nix -A sources.agent-js-monorepo --eval`
, agent-js-monorepo-src
, agentJsMonorepoTools ? import ./monorepo-tools.nix { inherit pkgs system; }
}:
let
  monorepo = pkgs.napalm.buildPackage agent-js-monorepo-src {
    name = "agent-js-monorepo";
    propagatedBuildInputs = [ (agentJsMonorepoTools agent-js-monorepo-src) ];
    outputs = [ "out" "lib" "agent" "bootstrap" ];
    configureScript = builtins.toFile "tmp-nix-configure.sh" ''
      export HOME=$(mktemp -d)
    '';
    npmCommands = [
      # Do this with --ignore-scripts to ensure we fetch deps, but don't trigger any npm scripts.
      # This is to allow for npm scripts that depend on dep's npm bin scripts.
      # Those scripts may have shebangs in them, and nix can only patchShebangs after each command.
      # So we fetch deps, let it patchShebangs, and then npm install again w/ postinstall scripts
      "npm install --ignore-scripts"
      "npm install"
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
      cp -R $out/node_modules $bootstrap/
      cp -R ./packages/bootstrap/* $bootstrap/
    '';
  };
in
monorepo
