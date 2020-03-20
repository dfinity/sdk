{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, RustSec-advisory-db ? null
, pkgs ? import ./nix { inherit system crossSystem config overlays RustSec-advisory-db; }
}:
rec {

  dfx = import ./dfx.nix { inherit pkgs userlib-js; };

  e2e-tests = import ./e2e/bats { inherit pkgs dfx; };
  e2e-tests-ic-ref = import ./e2e/bats { inherit pkgs dfx; use_ic_ref = true; };
  node-e2e-tests = import ./e2e/node { inherit pkgs dfx; };

  userlib-js = import ./src/userlib/js { inherit pkgs; };

  cargo-audit = import ./cargo-audit.nix { inherit pkgs; };

  inherit (pkgs) nix-fmt nix-fmt-check;

  public = import ./public { inherit pkgs src; };
  inherit (public) install-sh-release install-sh;

  # This is to make sure CI evaluates shell derivations, builds their
  # dependencies and populates the hydra cache with them. We also use this in
  # `shell.nix` in the root to provide an environment which is the composition
  # of all the shells here.
  shells = {
    js-user-library = import ./src/userlib/js/shell.nix { inherit pkgs userlib-js; };
    rust-workspace = dfx.shell;
  };

  dfx-release = pkgs.lib.mkRelease "dfx" pkgs.releaseVersion dfx.standalone "dfx";

  licenses = {
    dfx = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
  };
}
