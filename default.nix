{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, RustSec-advisory-db ? null
}:
let
  pkgs = import ./nix {
    inherit system crossSystem config overlays RustSec-advisory-db;
  };
in
{
  inherit (pkgs) dfinity-sdk;

  e2e-tests = import ./e2e { inherit pkgs; };

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
    js-user-library = import ./src/userlib/js/shell.nix { inherit pkgs; };
    rust-workspace = import ./dfx-shell.nix { inherit (pkgs.dfinity-sdk) rust-package; inherit pkgs; };
  };

  dfx-release = pkgs.lib.mkRelease "dfx" pkgs.releaseVersion pkgs.dfinity-sdk.packages.rust-workspace-standalone "dfx";
}
