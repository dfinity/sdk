# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, releaseVersion ? "latest"
  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? false
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

  sourcesnix = builtins.fetchurl {
    url = https://raw.githubusercontent.com/nmattia/niv/506b896788d9705899592a303de95d8819504c55/nix/sources.nix;
    sha256 = "007bgq4zy1mjnnkbmaaxvvn4kgpla9wkm0d3lfrz3y1pa3wp9ha1";
  };

  pkgs = import commonSrc {
    inherit system;
    overlays = [
      (
        self: super:
          let
            nixFmt = self.lib.nixFmt { root = ../.; };
          in
            {
              sources = super.sources // sources;

              inherit releaseVersion;

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

              lib = super.lib // {
                mk-jobset = import ./mk-jobset.nix self;

                mkRelease = super.callPackage ./mk-release.nix { inherit isMaster; };
              };

              # An attribute set mapping every supported system to a nixpkgs evaluated for
              # that system. Special care is taken not to reevaluate nixpkgs for the current
              # system because we already did that in self.
              pkgsForSystem = super.lib.genAttrs [ "x86_64-linux" "x86_64-darwin" ] (
                supportedSystem:
                  if supportedSystem == system
                  then self
                  else import ./. {
                    inherit releaseVersion isMaster RustSec-advisory-db;
                    system = supportedSystem;
                  }
              );
            }
      )
    ];
  };
in
pkgs
