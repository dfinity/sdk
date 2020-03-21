{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, releaseVersion ? "latest"
, RustSec-advisory-db ? null
, pkgs ? import ./nix { inherit system crossSystem config overlays releaseVersion RustSec-advisory-db; }
}:
let
  packages = rec {

    dfx = import ./dfx.nix { inherit pkgs userlib-js; };

    e2e-tests = import ./e2e/bats { inherit pkgs dfx; };
    node-e2e-tests = import ./e2e/node { inherit pkgs dfx; };

    userlib-js = import ./src/userlib/js { inherit pkgs; };

    cargo-audit = import ./cargo-audit.nix { inherit pkgs; };

    inherit (pkgs) nix-fmt nix-fmt-check;

    public = import ./public { inherit pkgs src; };
    inherit (public) install-sh-release install-sh;

    # This is to make sure CI evaluates shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      js-user-library = import ./src/userlib/js/shell.nix { inherit pkgs userlib-js; };
      rust-workspace = dfx.shell;
    };

    dfx-release = pkgs.lib.mkRelease "dfx" pkgs.releaseVersion dfx.standalone "dfx";

    licenses = {
      dfx = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
    };

    # Build a script to copy the dfx and install.sh release to the
    # dfinity-download S3 bucket which is the origin of the
    # download.dfinity.systems world-wide CDN.
    #
    # This script will be executed by DFINITY's Continuous Deployment
    # system. That system will also set the correct AWS credentials and the
    # DFINITY_DOWNLOAD_BUCKET environment variable.
    publish =
      let
        s3cp = pkgs.lib.writeCheckedShellScriptBin "s3cp" [] ''
          set -eu
          PATH="${pkgs.lib.makeBinPath [ pkgs.awscli ]}"
          pkg="$1"; file="$2"; dstDir="$3"; contentType="$4"; cacheControl="$5"
          aws s3 cp "$pkg/$file" "s3://$DFINITY_DOWNLOAD_BUCKET/$dstDir/$file" \
            --cache-control "$cacheControl" \
            --no-guess-mime-type --content-type "$contentType" \
            --no-progress
        '';
        packagesFor = wantedSystem:
          if system == wantedSystem
          then packages
          else
            # TODO: ci/ci.nix already evaluates ./. for all supported
            # systems so the following causes an unnecessary extra
            # evaluation. Let's refactor ci/ci.nix such that the
            # already evaluated jobsets for each system are available
            # to the jobsets themselves. This way we can speed up
            # evaluation on both CI and in nix-shells.
            import ./. {
              system = wantedSystem;
              inherit crossSystem config overlays src RustSec-advisory-db;
            };
        packages_x86_64-linux = packagesFor "x86_64-linux";
        packages_x86_64-darwin = packagesFor "x86_64-darwin";
      in
        {
          dfx = pkgs.lib.linuxOnly (
            pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
              set -eu
              PATH="${pkgs.lib.makeBinPath [ s3cp ]}"

              v="${pkgs.releaseVersion}"
              cache_long="max-age=31536000" # 1 year

              s3cp "${packages_x86_64-linux.dfx-release}"  "dfx-$v.tar.gz" "sdk/dfx/$v/x86_64-linux"  "application/gzip" "$cache_long"
              s3cp "${packages_x86_64-darwin.dfx-release}" "dfx-$v.tar.gz" "sdk/dfx/$v/x86_64-darwin" "application/gzip" "$cache_long"
            ''
          );
          install-sh = pkgs.lib.linuxOnly (
            pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
              set -eu
              PATH="${pkgs.lib.makeBinPath [ s3cp ]}"

              do_not_cache="max-age=0,no-cache"

              # TODO: I don't like not caching manifest.json and install.sh.
              # Consider configuring the S3 bucket as a website and
              # turning install.sh into an uncached (or shortly cached) redirect to a
              # cached versioned install.sh using the --website-redirect option.

              s3cp "${packages_x86_64-linux.install-sh-release}" "manifest.json" "sdk" "application/x-sh" "$do_not_cache"
              s3cp "${packages_x86_64-linux.install-sh-release}" "install.sh"    "sdk" "application/x-sh" "$do_not_cache"
            ''
          );
        };
  };
in
packages
