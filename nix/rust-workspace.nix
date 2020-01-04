{ motoko
, buildDfinityRustPackage
, cargo-graph
, darwin
, dfinity
, graphviz
, lib
, libressl
, runCommandNoCC
, stdenv
, dfinity-sdk
}:
let
  workspace = buildDfinityRustPackage {
    repoRoot = ../.;
    name = "dfinity-sdk-rust";
    srcDir = ../.;
    regexes = [
      ".*/assets/.*$"
      ".*\.rs$"
      ".*\.lalrpop$"
      ".*Cargo\.toml$"
      ".*Cargo\.lock$"
      "^.cargo/config$"
    ];
  };
  workspace' = (workspace //
    { lint = workspace.lint.overrideAttrs (oldAttrs: {
      nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
        cargo-graph
        graphviz
      ];

      postDoc = oldAttrs.postDoc + ''
        pushd src/dfx
        cargo graph | dot -Tsvg > ../../target/doc/dfx/cargo-graph.svg
        popd
      '';

      postInstall = oldAttrs.postInstall + ''
        echo "report cargo-graph-dfx $doc dfx/cargo-graph.svg" >> \
          $doc/nix-support/hydra-build-products
      '';
    });
  });
in

# override all derivations and add DFX_ASSETS as an environment variable
(lib.mapAttrs (k: drv:
  if !lib.isDerivation drv then drv else
    drv.overrideAttrs (_: {
      DFX_ASSETS = runCommandNoCC "dfx-assets" {} ''
        mkdir -p $out
        cp ${dfinity.nodemanager}/bin/nodemanager $out
        cp ${dfinity.ic-client}/bin/client $out
        cp ${motoko.moc-bin}/bin/moc $out
        cp ${motoko.mo-ide}/bin/mo-ide $out
        cp ${motoko.didc}/bin/didc $out
        cp ${motoko.rts}/rts/mo-rts.wasm $out
        mkdir $out/stdlib && cp -R ${motoko.stdlib}/. $out/stdlib
        mkdir $out/js-user-library && cp -R ${dfinity-sdk.packages.userlib.js}/. $out/js-user-library
      '';
    })
) workspace')
(throw "this argument is used to trigger the functor and shouldn't actually be evaluated.")
