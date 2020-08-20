{ system ? builtins.currentSystem
, src ? builtins.fetchGit ./.
, releaseVersion ? "latest"
, RustSec-advisory-db ? pkgs.sources.advisory-db
, pkgs ? import ./nix { inherit system; }
, jobset ? import ./ci/ci.nix { inherit system releaseVersion RustSec-advisory-db pkgs src; }
}:
rec {
  dfx = import ./dfx.nix { inherit pkgs agent-js assets; };

  e2e-tests = import ./e2e/bats { inherit pkgs dfx; };
  e2e-tests-ic-ref = import ./e2e/bats { inherit pkgs dfx; use_ic_ref = true; };
  node-e2e-tests = import ./e2e/node { inherit pkgs dfx; };

  # Agents in varous languages
  agent-js = import ./src/agent/javascript { inherit pkgs; };

  # Bootstrap frontend.
  bootstrap-js = import ./src/bootstrap { inherit pkgs agent-js; };

  cargo-audit = import ./cargo-audit.nix { inherit pkgs RustSec-advisory-db; };

  assets = import ./assets.nix { inherit pkgs bootstrap-js distributed-canisters; };

  distributed-canisters = import ./distributed-canisters.nix { inherit pkgs; };

  inherit (pkgs) nix-fmt nix-fmt-check;

  install = import ./public { inherit pkgs src; };

  # This is to make sure CI evaluates shell derivations, builds their
  # dependencies and populates the hydra cache with them. We also use this in
  # `shell.nix` in the root to provide an environment which is the composition
  # of all the shells here.
  shells = {
    js-user-library = import ./src/agent/javascript/shell.nix { inherit pkgs agent-js; };
    rust-workspace = dfx.shell;
  };

  licenses = {
    dfx = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
  };

  publish = import ./publish.nix {
    inherit pkgs releaseVersion;
    inherit (jobset) install;
    dfx = jobset.dfx.standalone;
  };
}
