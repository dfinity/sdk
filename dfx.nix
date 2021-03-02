# This file defines all flavors of the dfx build:
#   * lint and documentation
#   * debug build
#   * release build
#
# If you only intend to perform a release build, run:
#   nix-build ./dfx.nix -A build

{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
, assets ? import ./assets.nix { inherit pkgs; }
}:

let
  workspace =
    (
      pkgs.rustBuilder.overrideScope' (
        self_rs: super_rs: {
          inherit (pkgs.pkgsStatic.rustBuilder) makePackageSet;
        }
      )
    ).mkDfinityWorkspace {
      cargoFile = ./src/dfx/Cargo.nix;
      crateOverrides = [
        (
          pkgs.rustBuilder.rustLib.makeOverride {
            overrideAttrs = oldAttrs: {
              OPENSSL_STATIC = true;
              OPENSSL_LIB_DIR = "${pkgs.pkgsStatic.openssl.out}/lib";
              OPENSSL_INCLUDE_DIR = "${pkgs.pkgsStatic.openssl.dev}/include";
            };
          }
        )
        (
          pkgs.rustBuilder.rustLib.makeOverride {
            registry = "unknown";
            overrideAttrs = _: {
              DFX_ASSETS = assets;
            };
          }
        )
      ];
    };

  dfx = workspace.dfx.release;
in

dfx // {
  standalone = pkgs.lib.standaloneRust {
    drv = dfx;
    exename = "dfx";
    usePackager = false;
  };
  shell = workspace.shell;
}
