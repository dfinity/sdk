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

  e2e-tests = import ./e2e { inherit pkgs; };

  userlib-js = import ./src/userlib/js { inherit pkgs; };

  # The cargo audit job for known vulnerabilities. This generally run
  # against the advisory database pinned in sources.json; on Hydra
  # (master) however the latest advisory database is fetched from
  # RustSec/advisory-db. This means that whenever a new security
  # vulnerability is published or when Cargo.lock has been changed `cargo
  # audit` will run.
  cargo-audit = import ./cargo-audit.nix { inherit pkgs; };

  inherit (pkgs) nix-fmt nix-fmt-check;

  public = import ./public { inherit pkgs; };

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
    rust-workspace = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
  };
}
