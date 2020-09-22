{ system ? builtins.currentSystem
, pkgs ? import ./nix { inherit system; }
, src ? builtins.fetchGit ./.
}:

with pkgs.lib;

# This file collects all the crates in the "debug" workspace, runs clippy, cargo fmt, and puts all
# the produced binaries in $out/bin (so that the dependency report can check their contents).
let
  # clippy directly uses librustc, so our regular rustc wrapper is ignored.
  # this either should be moved into cargo2nix or i should expose the cargo2nix wrapper
  # function somewhere ergonomically.
  clippyWrapper = pkgs.runCommandNoCC "clippy-wrapper"
    {
      inherit (pkgs.stdenv) shell;
      exename = "clippy-driver";
      rustc = pkgs.clippy;
      utils = "${pkgs.sources.cargo2nix}/overlay/utils.sh";
    } ''
    mkdir -p $out/bin
    substituteAll ${pkgs.sources.cargo2nix}/overlay/wrapper.sh $out/bin/$exename
    chmod +x $out/bin/$exename
    # cargo-clippy runs 'exec $(dirname $0)/clippy-driver' so they need to be in the same place
    cp ${pkgs.clippy}/bin/cargo-clippy $out/bin
  '';

  lintCrate = drv: {
    name = "crate-${drv.name}-${drv.version}-lint";
    nativeBuildInputs = drv.nativeBuildInputs ++ [ pkgs.rustfmt clippyWrapper ];
    runCargo = ''
      cargo fmt --all -- --check
      cargo clippy $CARGO_VERBOSE --tests --benches -- -D warnings -D clippy::all -C debug-assertions=on
      cargo clippy $CARGO_VERBOSE --tests --benches -- -D warnings -D clippy::all -C debug-assertions=off
    '';
    installPhase = ''
      mkdir -p $out
    '';
  };

  runTests = drv:
    pkgs.runCommandNoCC "test-${drv.name}"
      (
        {
          LD = "${pkgs.stdenv.cc}/bin/ld";
          CARGO_PKG_NAME = drv.name;
          CARGO_MANIFEST_DIR = drv.src;
          IN_NIX_TEST = 1;
        } // addCanisterBins drv.name
      ) ''
      mkdir -p $out/${drv.name}
      export CARGO_TARGET_DIR=$out/${drv.name}
      for bin in ${drv}/bin/*; do
        # non-[[bin]] binaries are suffixed with a metadata hash
        if echo "$bin" | grep -qP -- '-[0-9a-f]{16}$'; then
          "$bin" --test
        fi
      done
    '';

  runBenchmarks = drv:
    pkgs.runCommandNoCC "bench-${drv.name}"
      (
        {
          LD = "${pkgs.stdenv.cc}/bin/ld";
          CARGO_PKG_NAME = drv.name;
          CARGO_MANIFEST_DIR = drv.src;
          IN_NIX_TEST = 1;
          nativeBuildInputs = [ pkgs.gnuplot ];
          # Benchmarks results of the master jobset are uploaded to
          # ElasticSearch. So we need to run those benchmarks on a dedicated and
          # idle builder to minimise noise in the results. However for PRs we
          # don't mind if benchmarks are run on regular builders. So we only
          # require the "benchmark" feature for builds on master to not cause a
          # queue of PR builds to form before the benchmark builder. In case an
          # engineer is interested in running the benchmarks of a PR on the dedicated
          # builder he can label the PR with `ci-run-benchmarks-on-dedicated-builder`.
          requiredSystemFeatures =
            pkgs.lib.optional (pkgs.isMaster || (pkgs.labels.ci-run-benchmarks-on-dedicated-builder or false)) [
              "benchmark"
            ];
        } // addCanisterBins drv.name
      ) ''
      mkdir -p $out/${drv.name}
      export CARGO_TARGET_DIR=$out/${drv.name}
      for bin in ${drv}/bin/*; do
        # non-[[bin]] binaries are suffixed with a metadata hash
        if echo "$bin" | grep -qP -- '-[0-9a-f]{16}$'; then
          "$bin" --bench
        fi
      done
    '';

  # we want to aggregate all the [lint,test,bench] jobs together in one derivation, but it should also be trivial for a dev to run `nix-build rs -A run-tests.$some-crate`.
  buildEnvWithPassthru = attrs: crates: pkgs.buildEnv
    (
      attrs // {
        passthru = crates;
        paths = attrValues crates;
      }
    );

in
{
  lint =
    buildEnvWithPassthru
      {
        name = "dfinity-rs-lint";
        pathsToLink = [ "/empty" ];
      }
      (pkgs.dfinity-foreach-crate (c: c.test.overrideDerivation lintCrate));

  tests = buildEnvWithPassthru
    {
      name = "dfinity-rs-tests";
    }
    (
      pkgs.dfinity-foreach-crate (
        x:
          if builtins.elem x.debug.name cratesWithReleaseTests
          then runTests x.test_release
          else runTests x.test
      )
    );

  benchmarks = pkgs.lib.runBenchmarks {
    results = buildEnvWithPassthru
      {
        name = "dfinity-rs-benches";
        postBuild = ''
          date --utc --iso-8601=seconds > $out/timestamp
        '';
        # lib.runBenchmarks searches directories called "target" for results
        extraPrefix = "/target";
      }
      (pkgs.dfinity-foreach-crate (x: runBenchmarks x.bench));
    inherit src;
    name = "workspace";
  };
}
