# This file defines all flavors of the dfx build:
#   * lint and documentation
#   * debug build
#   * release build
#
# If you only intend to perform a release build, run:
#   nix-build ./dfx.nix -A build

{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
}:
let
  lib = pkgs.lib;
  workspace = pkgs.buildDfinityRustPackage {
    repoRoot = ./.;
    name = "dfinity-sdk-rust";
    srcDir = ./.;
    regexes = [
      ".*/assets/.*$"
      ".*\.rs$"
      ".*\.lalrpop$"
      ".*Cargo\.toml$"
      ".*Cargo\.lock$"
      "^.cargo/config$"
    ];
    static = pkgs.stdenv.isLinux;
  };
  workspace' = (
    workspace // {
      lint = workspace.lint.overrideAttrs (
        oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
            pkgs.cargo-graph
            pkgs.graphviz
          ];

          postDoc = oldAttrs.postDoc + ''
            pushd src/dfx
            cargo graph | dot -Tsvg > \
              ../../target/$CARGO_BUILD_TARGET/doc/dfx/cargo-graph.svg
            popd
          '';

          postInstall = oldAttrs.postInstall + ''
            echo "report cargo-graph-dfx $doc dfx/cargo-graph.svg" >> \
              $doc/nix-support/hydra-build-products
          '';
        }
      );
    }
  );
in

  # override all derivations and add DFX_ASSETS as an environment variable
(
  lib.mapAttrs (
    k: drv:
      if !lib.isDerivation drv then drv else
        drv.overrideAttrs (
          _: {
            DFX_ASSETS = pkgs.runCommandNoCC "dfx-assets" {} ''
              mkdir -p $out
              cp ${pkgs.dfinity.nodemanager}/bin/nodemanager $out
              cp ${pkgs.dfinity.ic-client}/bin/client $out
              cp ${pkgs.motoko.moc-bin}/bin/moc $out
              cp ${pkgs.motoko.mo-ide}/bin/mo-ide $out
              cp ${pkgs.motoko.didc}/bin/didc $out
              cp ${pkgs.motoko.rts}/rts/mo-rts.wasm $out
              mkdir $out/stdlib && cp -R ${pkgs.motoko.stdlib}/. $out/stdlib
              mkdir $out/js-user-library && cp -R ${pkgs.dfinity-sdk.packages.userlib.js}/. $out/js-user-library
            '';
          }
        )
  ) workspace'
)
  (throw "this argument is used to trigger the functor and shouldn't actually be evaluated.")
