# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, isMaster ? true
, labels ? {}
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
    url = https://raw.githubusercontent.com/nmattia/niv/d13bf5ff11850f49f4282d59b25661c45a851936/nix/sources.nix;
    sha256 = "0a2rhxli7ss4wixppfwks0hy3zpazwm9l3y2v9krrnyiska3qfrw";
  };

  pkgs = import (commonSrc + "/pkgs") {
    inherit system isMaster labels;
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
              agent-js-monorepo = import ./agent-js/agent-js-monorepo.nix {
                inherit system pkgs;
                agent-js-monorepo-src = self.sources.agent-js-monorepo;
              };
              ic-ref = (import self.sources.ic-ref { inherit (self) system; }).ic-ref;

              nix-fmt = nixFmt.fmt;
              nix-fmt-check = nixFmt.check;

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
