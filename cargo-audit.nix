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
  # Ignore this vulnerability for as we have an indirect dependency on it
  # ID:       RUSTSEC-2020-0146
  # Crate:    generic-array
  # Version:  0.12.3
  # Date:     2020-04-09
  # URL:      https://rustsec.org/advisories/RUSTSEC-2020-0146
  # Title:    arr! macro erases lifetimes
  # Solution:  upgrade to >= 0.14.0
  # Dependency tree:
  # generic-array 0.12.3
  ignores = [ "RUSTSEC-2020-0146" ];
}
