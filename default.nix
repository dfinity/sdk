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
}
