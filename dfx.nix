# This file defines all flavors of the dfx build:
#   * lint and documentation
#   * debug build
#   * release build
#
# If you only intend to perform a release build, run:
#   nix-build ./dfx.nix -A build

{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
, userlib-js ? import ./src/userlib/js { inherit pkgs; }
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

    override = _: {
      RUST_TEST_THREADS = 1;
    };
  };

  # add extra executables used when linting
  addLintInputs = ws:
    ws // {
      lint = ws.lint.overrideAttrs (
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
    };

  # set DFX_ASSETS for the builds and shells
  addAssets = ws:
  # override all derivations and add DFX_ASSETS as an environment variable
    (
      lib.mapAttrs (
        k: drv:
          if !lib.isDerivation drv then drv else
            drv.overrideAttrs (
              _: {
                DFX_ASSETS = pkgs.runCommandNoCC "dfx-assets" {} ''
                  mkdir -p $out
                  cp ${pkgs.dfinity.ic-replica}/bin/replica $out
                  cp ${pkgs.motoko.moc-bin}/bin/moc $out
                  cp ${pkgs.motoko.mo-ide}/bin/mo-ide $out
                  cp ${pkgs.motoko.didc}/bin/didc $out
                  cp ${pkgs.motoko.rts}/rts/mo-rts.wasm $out
                  mkdir $out/stdlib && cp -R ${pkgs.motoko.stdlib}/. $out/stdlib
                  mkdir $out/js-user-library && cp -R ${userlib-js}/. $out/js-user-library
                '';
              }
            )
      ) ws
    );

  # add a `standalone` target stripped of nix references
  addStandalone = ws:
    ws // {
      standalone = pkgs.lib.standaloneRust
        {
          drv = ws.build;
          exename = "dfx";
          usePackager = false;
        };
    };

  # fixup the shell for more convenient developer use
  fixShell = ws:
    ws // {
      shell =
        pkgs.mkCompositeShell {
          name = "dfinity-sdk-rust-env";
          buildInputs = [ pkgs.rls ];
          inputsFrom = [ ws.shell ];
          shellHook = ''
            # Set CARGO_HOME to minimize interaction with any environment outside nix
            export CARGO_HOME=${if pkgs.lib.isHydra then "." else toString ./.}/.cargo-home

            # Set environment variable for debug version.
            export DFX_TIMESTAMP_DEBUG_MODE_ONLY=$(date +%s)
          '';
        };
    };

in
fixShell (
  addStandalone ((addLintInputs (addAssets workspace)))
    (throw "this argument is used to trigger the functor and shouldn't actually be evaluated.")
)
