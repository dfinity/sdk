# This file defines all flavors of the dfx build:
#   * lint and documentation
#   * debug build
#   * release build
#
# If you only intend to perform a release build, run:
#   nix-build ./dfx.nix -A build

{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
, agent-js ? import ./src/agent/javascript { inherit pkgs; }
, assets ? import ./assets.nix { inherit pkgs agent-js; }
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
    cargoTestCommands = _: [
      ''cargo $cargo_options test $cargo_test_options --workspace''
    ];
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
                DFX_ASSETS = assets;
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

  # Note that on Linux we need the static environment.
  cc = if pkgs.stdenv.isLinux
  then pkgs.pkgsStatic.stdenv.cc
  else pkgs.stdenv.cc;

  # fixup the shell for more convenient developer use
  fixShell = ws:
    ws // {
      shell =
        pkgs.mkCompositeShell {
          name = "dfinity-sdk-rust-env";
          nativeBuildInputs = [
            pkgs.rls
            # wabt-sys needs file in path, as well as cc (for cmake).
            pkgs.file
            cc
            pkgs.coreutils
          ] ++ lib.optional pkgs.stdenv.isDarwin pkgs.stdenv.cc.bintools;
          inputsFrom = [ ws.shell ];
          shellHook = ''
            # Set CARGO_HOME to minimize interaction with any environment outside nix
            export CARGO_HOME=${if pkgs.lib.isHydra then "." else toString ./.}/.cargo-home

            if [ ! -d "$CARGO_HOME" ]; then
                mkdir -p $CARGO_HOME
                echo "[net]
                      git-fetch-with-cli = true" > $CARGO_HOME/config
            fi

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
