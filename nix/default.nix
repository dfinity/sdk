# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem, isMaster ? true, labels ? {} }:
let
  # The `common` repo provides code (mostly Nix) that is used in the
  # `infra`, `dfinity` and `sdk` repositories.
  #
  # To conveniently test changes to a local `common` repo you set the `COMMON`
  # environment variable to an absolute path of it. For example:
  #
  #   COMMON="$(realpath ../common)" nix-build -A rust-workspace
  commonSrc = let
    localCommonSrc = builtins.getEnv "COMMON";
  in
    if localCommonSrc != "" then localCommonSrc else sources.common;

  sources = import sourcesnix {
    sourcesFile = ./sources.json;
    inherit pkgs;
  };

  sourcesnix = builtins.fetchurl {
    url =
      "https://raw.githubusercontent.com/nmattia/niv/d13bf5ff11850f49f4282d59b25661c45a851936/nix/sources.nix";
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
              agent-rs = self.naersk.buildPackage {
                name = "agent-rs";
                root = self.sources.agent-rs;
                cargoBuildOptions = x: x ++ [ "-p" "icx" ];
                cargoTestOptions = x: x ++ [ "-p" "icx" ];
                buildInputs = [ self.pkgsStatic.openssl self.pkg-config ]
                ++ self.lib.optional self.stdenv.isDarwin pkgs.libiconv;
                override = attrs: { OPENSSL_STATIC = "1"; };
              };
              icx-proxy = self.naersk.buildPackage {
                name = "icx-proxy";
                root = self.sources.icx-proxy;
                cargoBuildOptions = x: x ++ [ "-p" "icx-proxy" ];
                cargoTestOptions = x: x ++ [ "-p" "icx-proxy" ];
                buildInputs = [ self.pkgsStatic.openssl self.pkg-config ]
                ++ self.lib.optional self.stdenv.isDarwin pkgs.libiconv;
                override = attrs: { OPENSSL_STATIC = "1"; };
              };
              dfinity =
                (import self.sources.dfinity { inherit (self) system; }).dfinity.rs;
              napalm = self.callPackage self.sources.napalm {
                pkgs = self // { nodejs = self.nodejs-12_x; };
              };

              ic-ref = pkgs.runCommandNoCCLocal "ic-ref" {
                src = self.sources."ic-ref-${self.system}";
              } ''
                mkdir -p $out/bin
                tar -C $out/bin/ -xf $src
              '';
              motoko = pkgs.runCommandNoCCLocal "motoko" {
                src = self.sources."motoko-${self.system}";
              } ''
                mkdir -p $out/bin
                tar -C $out/bin -xf $src
              '';

              nix-fmt = nixFmt.fmt;
              nix-fmt-check = nixFmt.check;

              # An attribute set mapping every supported system to a nixpkgs evaluated for
              # that system. Special care is taken not to reevaluate nixpkgs for the current
              # system because we already did that in self.
              pkgsForSystem = super.lib.genAttrs [ "x86_64-linux" "x86_64-darwin" ]
                (
                  supportedSystem:
                    if supportedSystem == system then
                      self
                    else
                      import ./. { system = supportedSystem; }
                );
            }
      )
    ];
  };
in
pkgs
