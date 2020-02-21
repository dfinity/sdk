# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, releaseVersion ? "latest"
, RustSec-advisory-db ? null
}:
let
  # The `common` repo provides code (mostly Nix) that is used in the
  # `infra`, `dfinity` and `sdk` repositories.
  #
  # To conveniently test changes to a local `common` repo you set the `COMMON`
  # environment variable to an absolute path of it. For example:
  #
  #   COMMON="$(realpath ../common)" nix-build -A rust-workspace
  commonSrc =
    let
      localCommonSrc = builtins.getEnv "COMMON";
    in
      if localCommonSrc != ""
      then localCommonSrc
      else sources.common;

  sources = import sourcesnix { sourcesFile = ./sources.json; inherit pkgs; };

  sourcesnix = builtins.fetchurl
    https://raw.githubusercontent.com/nmattia/niv/506b896788d9705899592a303de95d8819504c55/nix/sources.nix;

  pkgs = import commonSrc {
    inherit system crossSystem config;
    overlays = [
      (
        self: super:
          let
            nixFmt = self.lib.nixFmt { root = ../.; };
            isMaster = super.isMaster or false;
          in
            {
              sources = super.sources // sources;

              inherit releaseVersion isMaster;

              # The RustSec-advisory-db used by cargo-audit.nix.
              # Hydra injects the latest RustSec-advisory-db, otherwise we piggy
              # back on the one defined in sources.json.
              RustSec-advisory-db =
                if ! isNull RustSec-advisory-db
                then RustSec-advisory-db
                else self.sources.advisory-db;

              motoko = import self.sources.motoko { system = self.system; };
              dfinity = (import self.sources.dfinity { inherit (self) system; }).dfinity.rs;
              napalm = self.callPackage self.sources.napalm {
                pkgs = self // { nodejs = self.nodejs-12_x; };
              };

              inherit (nixFmt) nix-fmt;
              nix-fmt-check = nixFmt.check;

              lib = super.lib // { mkRelease = super.callPackage ./mk-release.nix { inherit isMaster; }; };
            }
      )
    ] ++ overlays;
  };
in
pkgs
