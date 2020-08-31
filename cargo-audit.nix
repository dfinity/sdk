# The cargo audit job for known vulnerabilities. This generally runs
# against the advisory database pinned in sources.json; on Hydra
# (master) however the latest advisory database is fetched from
# RustSec/advisory-db. This means that whenever a new security
# vulnerability is published or when Cargo.lock has been changed `cargo
# audit` will run.
{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
, RustSec-advisory-db ? pkgs.sources.advisory-db
}:
pkgs.lib.cargo-security-audit {
  name = "dfinity-sdk";
  cargoLock = ./Cargo.lock;
  db = RustSec-advisory-db;
  ignores = [];
}
