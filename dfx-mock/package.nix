{ naersk, rustfmt, stdenv, lib, darwin, clang, cmake, python3, rustNightly, libressl, pkg-config, moreutils, cargo-graph, graphviz }:
let
  name = "dfinity-sdk-dfx";

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

  src = lib.sourceFilesByRegex (lib.gitOnlySource ./.) [
    ".*\.txt$"
    ".*\.rs$"
    ".*Cargo\.toml$"
    ".*Cargo\.lock$"
    "^.cargo/config$"
  ];
in
naersk.buildPackage src
{
  inherit name;
  cargo = rustNightly;
  rustc = rustNightly;

  # We add two extra checks to cargo test:
  #   * linting through clippy
  #   * formatting through rustfmt
  #       https://github.com/rust-lang/rustfmt/tree/1d19a08ed4743e3c95176fb639ebcd50f68a3313#checking-style-on-a-ci-server
  cargoTest =
    lib.concatStringsSep "\n" (
    lib.concatMap
    (str: [ "echo 'Running: ${str}'" str ])
    [ "cargo clippy -- -D clippy::all"
      "cargo test --$CARGO_BUILD_PROFILE"
      "cargo fmt --all -- --check"
    ]);

  override = oldAttrs: {

    buildPhase = timestampPhase (oldAttrs.buildPhase) (lib.hasPrefix "cargo");
    checkPhase = timestampPhase (oldAttrs.checkPhase) (lib.hasPrefix "cargo");
    docPhase = timestampPhase (oldAttrs.docPhase) (lib.hasPrefix "cargo");

    buildInputs = oldAttrs.buildInputs ++ [
      rustfmt
      libressl
      pkg-config
      moreutils
      cargo-graph
      graphviz
    ] ++ stdenv.lib.optional (stdenv.isDarwin) darwin.apple_sdk.frameworks.Security;

    nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
      clang
      cmake
      python3
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
    postDoc = ''
      cargo graph | dot -Tsvg > ./target/doc/dfx/cargo-graph.svg
    '';

    postInstall = ''
      # XXX: naersk forwards the whole ./target/ to the output. When using 'cargo
      # run' in the build this creates issues as ./target/ becomes too big.
      # In particular "checking for references to /build/ in ..." creates more
      # logging than Hydra can handle.
      rm -rf $out/target
      mkdir -p $doc/nix-support
      echo "report cargo-doc-dfx $doc ./index.html" >> \
        $doc/nix-support/hydra-build-products
      echo "report cargo-graph-dfx $doc ./cargo-graph.svg" >> \
        $doc/nix-support/hydra-build-products
    '';
  });
}
