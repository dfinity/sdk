# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
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

  pkgs = import (commonSrc + "/pkgs") {
    inherit system;
    extraSources = sources;
    repoRoot = ../.;
    overlays = [
      (
        self: super:
          let
            nixFmt = self.lib.nixFmt {};
          in
            {
              motoko = import self.sources.motoko { inherit (self) system; };
              dfinity = (import self.sources.dfinity { inherit (self) system; }).dfinity.rs;
              napalm = self.callPackage self.sources.napalm {
                pkgs = self // { nodejs = self.nodejs-12_x; };
              };
              ic-ref = (import self.sources.ic-ref { inherit (self) system; }).ic-ref;

              inherit (nixFmt) nix-fmt;
              nix-fmt-check = nixFmt.check;

              lib = super.lib // {
                mk-jobset = import ./mk-jobset.nix self;
              };

              # An attribute set mapping every supported system to a nixpkgs evaluated for
              # that system. Special care is taken not to reevaluate nixpkgs for the current
              # system because we already did that in self.
              pkgsForSystem = super.lib.genAttrs [ "x86_64-linux" "x86_64-darwin" ] (
                supportedSystem:
                  if supportedSystem == system
                  then self
                  else import ./. {
                    system = supportedSystem;
                  }
              );
            }
      )
    ];
  };
in
pkgs
