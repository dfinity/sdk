{ rustfmt
, lib
, darwin
, clang
, cmake
, sources
, python3
, rustPlatform
, rls
, rustc
, cargo
, cargo-deny
, deny-check
, clippy
, cargo-audit
, callPackage
, llvmPackages_9
, pkgsStatic
, pkgs
, file
, writeShellScript
, releaseTools
, substituteAll
}:
{ name
, srcDir ? null
, src ? let
    gitOnlySourceFilter = lib.gitOnlySourceFilter;
  in
    builtins.path
      {
        path = srcDir;
        filter = p: t:
          lib.sourceFilesByRegexFilter srcDir regexes p t && gitOnlySourceFilter p t;
        name = "${name}-src";
      }
, regexes ? [
    ".*\.rs$"
    ".*Cargo\.toml$"
    ".*Cargo\.lock$"
    ".*\.wat$"
    "^rustfmt\.toml$"

    # even though this is not used on CI, we _do_ include it here. An error
    # will be thrown later in the build if there is indeed a config, notifying
    # the developer that all the configuration should go through environment
    # variables.
    "^.cargo/config$"
  ]
  # When disabled the `build` and `debug` derivations won't run their tests
  # which is handy when one needs to build a binary quickly. Note that the tests
  # of the "deps" derivation will build regardless of this setting which means
  # that when `doCcheck = false` the `build` and `debug` derivations can use the
  # same "deps" derivation as they would use when `doCheck = true` allowing them
  # to benefit from the cache.
, doCheck ? true
, cargoTestCommands ? default: default
, cargoBuild ? default: default
, override ? _oldAttrs: {}
  #, crateOverrides ? {}
}:

let
  inherit (pkgs.stdenv) isLinux;
  stdenv = pkgs.stdenv;

  # makes sure that doCheck is set. `mkDerivation` intercepts `doCheck` and
  # sets it back to `false` if `buildPlatform != hostPlatform`.
  forceDoCheck = oldAttrs: {
    enableDoCheck = "doCheck='yes please'";
    postConfigure = (oldAttrs.postConfigure or "") + ''
      eval "$enableDoCheck"
    '';
  };
  mergeForceDoCheck = attrs: attrs // forceDoCheck attrs;
  # When `doCheck = false` this disables the tests in the final derivation but
  # it won't change the tests of the "deps" derivation.
  handleDoCheck = drv:
    drv.overrideAttrs
      (
        oldAttrs: lib.optionalAttrs (!doCheck) {
          inherit doCheck;
          enableDoCheck = "";
        }
      );

  naersk = callPackage sources.naersk { inherit stdenv; };

  # the src and root attributes
  # src is used as source during the build, root is a path used to read the
  # Cargo.{lock,toml}.
  sr = { inherit src; } // lib.optionalAttrs (!isNull srcDir) { root = srcDir; };

  # Best approximation to a local "path". Fallback to src if srcDir isn't set.
  root = if isNull srcDir then src else srcDir;

  # Copy the default naersk build command and add --locked to ensure Cargo.lock
  # is up-to-date and --all-targets to make sure that benchmarks are built as
  # well.
  cargoBuildOptions = opts: opts ++ [ "--locked" "--all-targets" ];

  # The environment variables set during build.
  buildEnv = { failOnWarnings ? true }:
    let
      # Turn a platform config into a string suitable for use in an environment variable name.
      # Also note that CARGO_TARGET_* variables *also* need to be uppercased. No others do.
      # The cc crate actually can read env vars that have hyphens in the name, but in certain
      # cases, macOS can't preserve them in subshells.
      normalize = builtins.replaceStrings [ "-" ] [ "_" ];
      buildCC = pkgs.stdenv.cc;
      buildCfg = normalize stdenv.buildPlatform.config;

      # Makes sure that rustc fails on warnings.
      # Cargo specifies --cap-lints=allow to rustc when building external
      # dependencies, so this only impacts crates in the workspace.
      # See also:
      #   * https://github.com/rust-lang/cargo/issues/5998#issuecomment-419718613
      #   * https://doc.rust-lang.org/rustc/lints/levels.html
      #   * https://doc.rust-lang.org/rustc/lints/levels.html#capping-lints
      commonRustflags = lib.optionals failOnWarnings [ "-D" "warnings" ]
      ++ [ "-W" "rust-2018-idioms" ];
    in
      {
        "CC_${buildCfg}" = "${buildCC}/bin/cc";
        "CXX_${buildCfg}" = "${buildCC}/bin/c++";
        "AR_${buildCfg}" = "${buildCC.bintools.bintools}/bin/ar";
        "CARGO_TARGET_${lib.toUpper buildCfg}_LINKER" = "${buildCC}/bin/cc";

        # Indicate to the 'openssl' Rust crate that OpenSSL/LibreSSL shall be linked statically rather than dynamically.
        # See https://crates.io/crates/openssl -> Documentation -> Manual -> OPENSSL_STATIC for more details.
        OPENSSL_STATIC = true;

        OPENSSL_LIB_DIR = "${pkgsStatic.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgsStatic.openssl.dev}/include";

        CARGO_BUILD_RUSTFLAGS = commonRustflags;

        # Since we don't (want to) build rust-lld, we need to provide a linker.
        # We use LLVM's because it supports wasm and it's fast.
        # (note: we need LLVM 8+ for a working lld)
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${llvmPackages_9.lld}/bin/lld";
      } // lib.optionalAttrs isLinux (
        let
          hostTriple = stdenv.hostPlatform.config;
          hostCfg = normalize hostTriple;
          hostCC = stdenv.cc;
        in
          {
            CARGO_BUILD_TARGET = hostTriple;

            "CARGO_TARGET_${lib.toUpper hostCfg}_LINKER" = "${hostCC}/bin/${hostCC.targetPrefix}cc";
            "CC_${hostCfg}" = "${hostCC}/bin/${hostCC.targetPrefix}cc";
            "CXX_${hostCfg}" = "${hostCC}/bin/${hostCC.targetPrefix}c++";
            "AR_${hostCfg}" = "${hostCC.bintools.bintools}/bin/${hostCC.targetPrefix}ar";

            "CARGO_TARGET_${lib.toUpper hostCfg}_RUSTFLAGS" = [
              # Export symbols from lucet-runtime for objects to reenter
              "-C"
              "link-args=-Wl,--export-dynamic"
            ]
            # Careful! Cargo does not combine RUSTFLAGS from different sources. Only the most
            # specific one is used.
            ++ commonRustflags;
          }
      );
  #// lib.optionalAttrs (crateOverrides != {}) {
  #RUSTC_WRAPPER = substituteAll {
  #src = ./rustc-wrapper.sh;
  #inherit (stdenv) shell;
  #isExecutable = 1;
  #applyToTests = crateOverrides.patchTests or true;
  #extraFlags = crateOverrides.flags or "";
  #crateNames = lib.concatStringsSep "|" crateOverrides.crates;
  #};
  #};

  ## File checks

  # Check that all crates are present in the virtual manifest
  workspaceCratesCheck =
    let
      cargotoml = builtins.fromTOML (builtins.readFile (root + "/Cargo.toml"));
      workspaceMembers = sort (cargotoml.workspace.members or []);
      localCrates =
        sort
          (
            builtins.attrNames
              (
                lib.filterAttrs
                  (name: type: type == "directory" && isCrateDir name)
                  (builtins.readDir root)
              )
          );
      isCrateDir = dir: builtins.hasAttr "Cargo.toml"
        (builtins.readDir (root + "/${dir}"));
      sort = lib.sort (x: y: x > y);

      # NOTE: by performing the check on the diff, we make sure that we do
      # _not_ fail if there are more crates in the workspace than in the
      # directory. Cargo will most likely fail anyway, and this is not what we
      # test for.
      diff = lib.subtractLists workspaceMembers localCrates;
      areSame = if diff == [] then "yes" else "no";
    in
      ''
        if [ "${areSame}" != "yes" ]; then
          echo "Please add all crates to Cargo.toml!"
          echo "Workspace members:"
          echo "    ${toString workspaceMembers}"
          echo "Crates:"
          echo "    ${toString localCrates}"
          echo "Please add the following crates to the workspace:"
          echo "    ${toString diff}"
          exit 1
        fi
      '';

  # For finer by-platform and by-target control we specify all the
  # configuration through environment variables. To ensure developers are
  # not lead to believe their .cargo/config is taken into account we fail
  # loudly if one is found.
  cargoConfigCheck =
    ''
      if [ -f .cargo/config ]; then
          echo "All configuration should be done through environment variables"
          echo "Found configuration:"
          cat .cargo/config
          exit 2
      fi
    '';

  ## The various derivations

  # The main build
  drvBuild =
    naersk.buildPackage (
      sr // {
        inherit name rustc cargoTestCommands cargoBuildOptions cargoBuild;
        release = true;
        doCheck = true;
        doDoc = false;
        override = oldAttrs: let
          attrs = mergeForceDoCheck (
            {
              postConfigure =
                ''
                  # the `file` executable is needed in `wabt-sys`'s `cmake` build
                  export PATH="${file}/bin:$PATH"
                '';
            } // buildEnv {}
          );
        in
          attrs // override (oldAttrs // attrs);
      }
    );

  # The main build with symbols
  # We don't run tests, as they are ran in drvBuild
  drvBuildWithSymbols =
    naersk.buildPackage (
      sr // {
        inherit name rustc cargoTestCommands cargoBuildOptions cargoBuild;
        release = true;
        doCheck = false;
        doDoc = false;
        override = oldAttrs: let
          attrs = (
            {
              postConfigure =
                ''
                  # the `file` executable is needed in `wabt-sys`'s `cmake` build
                  export PATH="${file}/bin:$PATH"
                '';
            } // buildEnv {}
          );
        in
          attrs // override (oldAttrs // attrs) // {
            CARGO_BUILD_RUSTFLAGS = attrs.CARGO_BUILD_RUSTFLAGS ++ [ "-g" ];
          };
      }
    );

  # The debug build
  drvDebug = naersk.buildPackage (
    sr // {
      inherit rustc cargoBuildOptions cargoBuild;
      name = "${name}-debug";
      release = false;
      doDoc = false;
      override = oldAttrs: let
        attrs = {
          postConfigure =
            ''
              # the `file` executable is needed in `wabt-sys`'s `cmake` build
              export PATH="${file}/bin:$PATH"
            '';
        } // buildEnv { failOnWarnings = false; };
      in
        attrs // override (oldAttrs // attrs);
    }
  );

  # Derivation used for linting the code and doc
  drvLintAndDoc = (
    naersk.buildPackage (
      sr // {
        inherit rustc;
        name = "${name}-lint-and-doc";
        release = false;
        doCheck = true;
        cargoTestCommands = _: [ "cargo clippy --tests --benches -- -D clippy::all" ];
        doDoc = true;

        override = oldAttrs: let
          attrs = mergeForceDoCheck (
            {
              nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ clippy ];
              postConfigure =
                ''
                  # the `file` executable is needed in `wabt-sys`'s `cmake` build
                  export PATH="${file}/bin:$PATH"
                '';
            } // buildEnv {}
          );
        in
          attrs // override (oldAttrs // attrs);
      }
    )
  ).overrideAttrs (
    oldAttrs: {

      buildPhase = ''
        runHook preBuild
        echo no build
        runHook postBuild
      '';

      # as cargo-doc doesn't create a proper index page, we create one.
      postDoc = ''
        crates_table="$(cargo metadata --format-version=1 | ${pkgs.jq}/bin/jq -r -f ${./crates.jq} --arg pwd "$(pwd)")"

        # Create a nice homepage for the docs. It's awful that we have to copy the
        # HTML template like this, but the upstream issue [0] that would resolve this is
        # now five years old and doesn't look close to resolution.
        # [0]: https://github.com/rust-lang/cargo/issues/739
        substitute ${./index.html.template} target/doc/index.html --subst-var crates_table

        # Make the logo link to the nice homepage we just created. Otherwise it just
        # links to the root of whatever crate you happen to be looking at.
        cat >> target/doc/main.js <<EOF
        ;
        var el = document.querySelector("img[alt=logo]").closest("a");
        if (el.href != "index.html") {
            el.href = "../index.html";
        }
        EOF
      '';

      # We make the doc available as a build product.
      postInstall = ''
        mkdir -p $out # otherwise nothing is created for 'lint'
        mkdir -p $doc/nix-support
        echo "report cargo-doc-dfinity $doc index.html" >> \
          $doc/nix-support/hydra-build-products
      '';
    }
  );

  # Final fixup for the derivations:
  #   * timestamp interesting phases
  #   * enable RUST_BACKTRACE after preCheck
  #   * touch rust files, because cargo uses mtime
  fixupDrv = drv:
    drv.overrideAttrs
      (
        oldAttrs:
          lib.timestampPhases
            [ "buildPhase" "checkPhase" "installPhase" "docPhase" ]
            oldAttrs // {
            preCheck = (oldAttrs.preCheck or "") + "export RUST_BACKTRACE=1";
            # change mtime on all source .rs files
            # target/ already contains .rs files from dependencies, which are checked by
            # cargo fingerprinting, so they shouldn't be updated
            preBuild = ''
              find . -path ./target -prune -o -type f -print0 | xargs -0 touch
            '' + (oldAttrs.preBuild or "");
          }
      );
in

  # This is the final attribute set we return. It comprises build (+debug),
  # formatting, linting, documentation and some checks we perform on files.
  #
  # Additionally we add an "allChecks" attribute which builds all other
  # derivations. To make sure that "allChecks" builds the correct derivations
  # (e.g. if some were overriden later on) we make it a functor. This way
  # "allChecks" always refers to the final derivations.
  #
  # Finally we add a "shell" attribute, based on the "debug" attribute, but with
  # a few tweaks.

{
  # The actual build
  build = fixupDrv (handleDoCheck drvBuild);

  # The debug build
  debug = fixupDrv (handleDoCheck drvDebug);

  # The release build with symbols
  buildWithSymbols = fixupDrv drvBuildWithSymbols;

  # The lint and doc (expanded to "doc" in functor)
  lint = fixupDrv drvLintAndDoc;

  # Some checks that we run on the cargo config and Cargo.toml
  fileChecks = stdenv.mkDerivation
    {
      inherit src;
      name = "${name}-file-checks";
      phases = [ "unpackPhase" "checkPhase" "installPhase" ];
      doCheck = true;
      checkPhase = lib.concatStringsSep "\n"
        [
          cargoConfigCheck
          workspaceCratesCheck
        ];
      installPhase = "touch $out";
    };

  # Formatting check. Technically this doesn't need to be a naersk
  # derivation, but we get cargo for free.
  fmt = naersk.buildPackage (
    sr // {
      inherit rustc;
      singleStep = true;
      doDoc = false;
      name = "${name}-fmt";
      cargoBuild = _: "echo no build";
      cargoTestCommands = _: [ "cargo fmt --all -- --check" ];
      doCheck = true;
      override = oldAttrs: {
        installPhase = "mkdir $out";
        nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ rustfmt ];
      } // forceDoCheck oldAttrs;
    }
  );

  cargoLicenseReport = naersk.buildPackage (
    sr // {
      inherit src;
      name = "${name}-cargo-license-report";
      doDoc = false;
      cargoBuild = _: "true";
      doCheck = false;
      singleStep = true;
      override = oldAttrs: {
        buildCargoTable = ''
          to_entries
            | .[]
              # filter out local dependencies
            | select(.key | contains("path+") | not)
            | "<tr>
                <td>\(.key | sub(" \\(registry[^)]+\\)";""))</td>
                <td>\(.value.licenses | sort | join(", "))</td>
              </tr>"
        '';
        installPhase = ''
          mkdir -p $out/nix-support
          export licenseReport=$(${cargo-deny}/bin/cargo-deny list -f json -l crate | jq -r "$buildCargoTable")
          name=${name} substituteAll ${./licenses.html.in} $out/licenses.html
          cat <<EOF >>$out/nix-support/hydra-build-products
          report licenses $out/licenses.html
          EOF
        '';
      };
    }
  );

  # this needs to be a naersk build since it manages all the dependencies for us.
  # cargo-deny uses the sources that we provide through source replacement
  cargoLicenseCheck = naersk.buildPackage (
    sr // {
      inherit src;
      name = "${name}-cargo-license-check";
      doDoc = false;
      cargoBuild = _: "true";
      cargoTestCommands = _: [ "${deny-check}/bin/deny-check -h" ];
      doCheck = true;
      singleStep = true;
      override = _: { installPhase = "mkdir $out"; };
    }
  );

  # Add allChecks and shell based on final attributes.
  __functor = self: _:
    let
      jobs = removeAttrs self [ "__functor" ];
    in
      jobs // {
        allChecks = releaseTools.aggregate
          {
            name = "${name}-all-checks";
            constituents = builtins.attrValues jobs;
          };

        # Add attribute for "doc" based on the lint job
        doc = jobs.lint.doc;

        # use the debug derivation as template for the shell, but:
        #   * remove naersk references to prebuilt dependencies
        #   * add rustfmt and clippy
        #   * add shell hooks
        shell = jobs.debug.overrideAttrs (
          oldAttrs:
            {
              unpackPhase = "";
              configurePhase = "";
              buildPhase = "";
              checkPhase = "";
              installPhase = "";
              cargoconfig = "";
              builtDependencies = "";
              # this makes sure we don't depend on the crate sources in the
              # shell: https://dfinity.atlassian.net/browse/INF-1115
              crate_sources = "";
              nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
                rustfmt
                clippy
                cargo-audit
                rls
              ];
              RUST_SRC_PATH = rustPlatform.rustcSrc;

              # Make sure our specified rustc is in front of the PATH so cargo will
              # use.
              shellHook = ''
                export PATH="${rustc}/bin''${PATH:+:}$PATH"
              '';
            }
        );
      };
}
