{ release ? true
, doClippy ? false
, doFmt ? false
, doDoc ? false
, motoko
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
  DFX_ASSETS = runCommandNoCC "dfx-assets" {} ''
    mkdir -p $out
    cp ${dfinity.nodemanager}/bin/nodemanager $out
    cp ${dfinity.ic-client}/bin/client $out
    cp ${motoko.moc-bin}/bin/moc $out
    cp ${motoko.mo-ide}/bin/mo-ide $out
    cp ${motoko.didc}/bin/didc $out
    cp ${motoko.rts}/rts/mo-rts.wasm $out
    mkdir $out/stdlib && cp -R ${motoko.stdlib}/. $out/stdlib
  '';

  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ lib.optionals doDoc [
    cargo-graph
    graphviz
  ] ++ [stdenv.cc];

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
