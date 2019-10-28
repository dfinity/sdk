{ release ? true
, doClippy ? false
, doFmt ? false
, doDoc ? false
, actorscript
, buildDfinityRustPackage
, cargo-graph
, darwin
, dfinity
, graphviz
, lib
, libressl
, runCommandNoCC
, stdenv
}:
let
  drv = buildDfinityRustPackage {
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
    inherit release doClippy doFmt doDoc;
  };
in
drv.overrideAttrs (oldAttrs: {
  # The nodemanager does not have a standalone build, but the client does and we need it.
  DFX_ASSETS = runCommandNoCC "dfx-assets" {} ''
    mkdir -p $out
    cp ${if release then dfinity.rust-workspace else dfinity.rust-workspace-debug}/bin/nodemanager $out
    cp ${if release then dfinity.ic-client else dfinity.rust-workspace-debug}/bin/client $out
    cp ${actorscript.asc-bin}/bin/asc $out
    cp ${actorscript.as-ide}/bin/as-ide $out
    cp ${actorscript.didc}/bin/didc $out
    cp ${actorscript.rts}/rts/as-rts.wasm $out
    mkdir $out/stdlib && cp -R ${actorscript.stdlib}/. $out/stdlib
  '';

  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ lib.optionals doDoc [
    cargo-graph
    graphviz
  ];

  postDoc = oldAttrs.postDoc + ''
    pushd dfx
    cargo graph | dot -Tsvg > ../target/doc/dfx/cargo-graph.svg
    popd
  '';

  postInstall = oldAttrs.postInstall + lib.optionalString doDoc ''
    echo "report cargo-graph-dfx $doc dfx/cargo-graph.svg" >> \
      $doc/nix-support/hydra-build-products
  '';
})
