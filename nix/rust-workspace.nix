{ buildDfinityRustPackage, stdenv, lib, darwin, libressl, cargo-graph, graphviz
#, cargo-graph, graphviz
, dfinity, actorscript, runCommandNoCC
, release ? true # is it a "release" build, as opposed to "debug" ?
, doClippy ? false
, doFmt ? false
, doDoc ? false
}:
let
  drv = buildDfinityRustPackage {
    name = "dfinity-sdk-rust";
    srcDir = ../.;
    regexes = [
      ".*/assets/.*$"
      ".*\.rs$"
      ".*Cargo\.toml$"
      ".*Cargo\.lock$"
      "^.cargo/config$"
    ];
    inherit release doClippy doFmt doDoc;

    override = oldAttrs: {
      buildInputs = oldAttrs.buildInputs ++ [
        libressl.dev
      ] ++ lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security;

      # Indicate to the 'openssl' Rust crate that OpenSSL/LibreSSL shall be linked statically rather than dynamically.
      # See https://crates.io/crates/openssl -> Documentation -> Manual -> OPENSSL_STATIC for more details.
      OPENSSL_STATIC = true;
    };
  };
in
drv.overrideAttrs (oldAttrs: {
  DFX_ASSETS = runCommandNoCC "dfx-assets" {} ''
    mkdir -p $out
    cp ${dfinity.rust-workspace}/bin/{client,nodemanager} $out
    cp ${actorscript.asc}/bin/asc $out
    cp ${actorscript.as-ide}/bin/as-ide $out
    cp ${actorscript.didc}/bin/didc $out
    cp ${actorscript.rts}/rts/as-rts.wasm $out
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
