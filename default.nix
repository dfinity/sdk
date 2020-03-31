{ system ? builtins.currentSystem
, src ? null
, releaseVersion ? "latest"
  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? false
, RustSec-advisory-db ? null
, pkgs ? import ./nix { inherit system RustSec-advisory-db; }
, jobset ? import ./ci/ci.nix { inherit system releaseVersion RustSec-advisory-db pkgs isMaster src; }
}:
rec {
  dfx = import ./dfx.nix { inherit pkgs userlib-js; };

  e2e-tests = import ./e2e/bats { inherit pkgs dfx; };
  node-e2e-tests = import ./e2e/node { inherit pkgs dfx; };

  userlib-js = import ./src/userlib/js { inherit pkgs; };

  cargo-audit = import ./cargo-audit.nix { inherit pkgs; };

  inherit (pkgs) nix-fmt nix-fmt-check;

  public = import ./public { inherit pkgs src releaseVersion isMaster; };
  inherit (public) install-sh-release install-sh;

  # This is to make sure CI evaluates shell derivations, builds their
  # dependencies and populates the hydra cache with them. We also use this in
  # `shell.nix` in the root to provide an environment which is the composition
  # of all the shells here.
  shells = {
    js-user-library = import ./src/userlib/js/shell.nix { inherit pkgs userlib-js; };
    rust-workspace = dfx.shell;
  };

  dfx-release = pkgs.lib.mkRelease "dfx" releaseVersion dfx.standalone "dfx";

  licenses = {
    dfx = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
  };

  publish = import ./publish.nix {
    inherit pkgs releaseVersion;
    inherit (jobset) dfx-release install-sh-release;
  };
}
