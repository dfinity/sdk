{ naersk
, rustfmt
, rls
, stdenv
, lib
, darwin
, clang
, cmake
, python3
, rustPackages
, libressl
, pkg-config
, moreutils
, cargo-graph
, graphviz
, actorscript
, dfinity
, runCommand
, release ? true # is it a "release" build, as opposed to "debug" ?
, doClippy ? false
, doFmt ? false
, doDoc ? false
}:

let
  name = "dfinity-sdk-rust";

  # Neither Nix nor Hydra provide timestamps during build, which makes it
  # difficult to figure out why a particular build takes time.
  # Instead of fixing Nix and/or Hydra we simply tweak key commands (e.g.
  # `cargo build`) by piping the output through `ts` (e.g. `cargo build | ts`)
  # to log the timestamps on every line.
  #
  # phase: the content of the phase to patch
  # pred: the predicate for lines to tweak
  timestampPhase = phase: pred:
    let
      lines = lib.splitString "\n" phase;
      timestamp = line:
        if pred line
        then line + " 2>&1 | ts '[%Y-%m-%d %H:%M:%S]'"
        else line;
    in lib.concatMapStringsSep "\n" timestamp lines;

  src = lib.sourceFilesByRegex (lib.gitOnlySource ../.) [
    ".*/assets/.*$"
    ".*\.rs$"
    ".*Cargo\.toml$"
    ".*Cargo\.lock$"
    "^.cargo/config$"
  ];

  cargo = rustPackages.cargo;
  rustc = rustPackages.rustc;
in
naersk.buildPackage src {
  inherit name rustc release;

  # We add two extra checks to cargo test:
  #   * linting through clippy
  #   * formatting through rustfmt
  #       https://github.com/rust-lang/rustfmt/tree/1d19a08ed4743e3c95176fb639ebcd50f68a3313#checking-style-on-a-ci-server
  cargoTestCommands =
      [ ''cargo test "''${cargo_release[*]}"  -j $NIX_BUILD_CORES'' ]
      ++ lib.optional doClippy "cargo clippy --tests -- -D clippy::all"
      ++ lib.optional doFmt "cargo fmt --all -- --check";

  override = oldAttrs: {

    buildPhase = timestampPhase (oldAttrs.buildPhase) (lib.hasPrefix "cargo");
    checkPhase = timestampPhase (oldAttrs.checkPhase) (lib.hasPrefix "cargo");
    docPhase = timestampPhase (oldAttrs.docPhase) (lib.hasPrefix "cargo");

    buildInputs = oldAttrs.buildInputs ++ [
      rls
      rustfmt
      libressl
      pkg-config
      moreutils
      # cargo-graph
      # graphviz
    ] ++ stdenv.lib.optional (stdenv.isDarwin) darwin.apple_sdk.frameworks.Security;

    nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
      clang
      cmake
      python3
      rustPackages.clippy
    ];

    # Indicate to the 'openssl' Rust crate that OpenSSL/LibreSSL shall be linked statically rather than dynamically.
    # See https://crates.io/crates/openssl -> Documentation -> Manual -> OPENSSL_STATIC for more details.
    OPENSSL_STATIC = true;

    # Without this .cargo-home is used
    CARGO_HOME = ".cargo";

    # Makes sure that rustc fails on warnings.
    # Cargo specifies --cap-lints=allow to rustc when building external
    # dependencies, so this only impacts crates in the workspace.
    # See also:
    #   * https://github.com/rust-lang/cargo/issues/5998#issuecomment-419718613
    #   * https://doc.rust-lang.org/rustc/lints/levels.html
    #   * https://doc.rust-lang.org/rustc/lints/levels.html#capping-lints
    #
    # XXX: Some rustflags are specified in .cargo/config which are needed
    # during both development and CI runs (on Linux). However environment variable flags
    # (like RUSTFLAGS=-D warnings) take precedence over the ones in
    # .cargo/config, so we cannot fail-on-warning through environment
    # variables. We cannot specify -D warnings in .cargo/config either because
    # that would be a pain during local development. Instead we tweak the
    # .cargo/config before we kickstart the build.
    postConfigure = ''
      find .
      sed -i "s:\(rustflags = .*\)]:\1, \"-D\", \"warnings\"]:g" .cargo/config
    '';

  } // (
  # HACK:
  # The 'override' is applied to all builds performed by naersk:
  #   * The first "deps-only" build
  #   * The build of the actual package
  # We add an extra step to the workspace build (as opposed to the deps-only
  # build) to export the documentation. We do this by checking the name of the
  # derivation (the deps-only build has name "${name}-deps").
  lib.optionalAttrs (oldAttrs.name == name)
  {
    DFX_ASSETS = runCommand "dfx-assets" {} ''
      mkdir -p $out
      cp ${dfinity.rust-workspace}/bin/{client,nodemanager} $out
      cp ${actorscript.asc}/bin/asc $out
      cp ${actorscript.as-ide}/bin/as-ide $out
      cp ${actorscript.didc}/bin/didc $out
      cp ${actorscript.rts}/rts/as-rts.wasm $out
    '';

    ## This crashes when running locally, because Cargo.lock does not have a root
    ## table. Since we have a very simple graph and don't really need this it's
    ## best to ignore it for now.
    # postDoc = ''
    #   cargo graph | dot -Tsvg > ./target/doc/dfx/cargo-graph.svg
    # '';

    postInstall = ''
      # XXX: naersk forwards the whole ./target/ to the output. When using 'cargo
      # run' in the build this creates issues as ./target/ becomes too big.
      # In particular "checking for references to /build/ in ..." creates more
      # logging than Hydra can handle.
      rm -rf $out/target
    '' + lib.optionalString doDoc ''
      mkdir -p $doc/nix-support
      echo "report cargo-doc-dfinity $doc index.html" >> \
        $doc/nix-support/hydra-build-products
      echo "report cargo-graph-dfinity $doc ic_client/cargo-graph.svg" >> \
        $doc/nix-support/hydra-build-products
    '';
  });
}
