{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
}:
pkgs.lib.cargo-security-audit {
  name = "dfinity-sdk";
  cargoLock = ./Cargo.lock;
  db = pkgs.RustSec-advisory-db;
  ignores = [];
}
