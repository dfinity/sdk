# Build a script to copy the dfx and install.sh release to the
# dfinity-download S3 bucket which is the origin of the
# download.dfinity.systems world-wide CDN.
#
# This script will be executed by DFINITY's Continuous Deployment
# system. That system will also set the correct AWS credentials and the
# DFINITY_DOWNLOAD_BUCKET environment variable.
{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, releaseVersion ? "latest"
, RustSec-advisory-db ? null
, pkgs ? import ./nix { inherit system crossSystem config overlays releaseVersion RustSec-advisory-db; }
, packages ? import ./. { inherit system crossSystem config overlays releaseVersion RustSec-advisory-db pkgs; }
}:
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
      PATH="${pkgs.lib.makeBinPath [ s3cp pkgs.jo pkgs.curl pkgs.coreutils ]}"

      v="${pkgs.releaseVersion}"
      cache_long="max-age=31536000" # 1 year

      file="dfx-$v.tar.gz"
      dir="sdk/dfx/$v"

      s3cp "${packages_x86_64-linux.dfx-release}" "$file" "$dir/x86_64-linux" "application/gzip" "$cache_long"
      s3cp "${packages_x86_64-darwin.dfx-release}" "$file" "$dir/x86_64-darwin" "application/gzip" "$cache_long"

      msg=$(cat <<EOI
      DFX-$v has been published to DFINITY's CDN at:
      * https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-linux/$file
      * https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-darwin/$file
      Install the SDK by following the instructions on: https://sdk.dfinity.org/docs/download.html.
      EOI
      )
      jo "text=$msg" \
        | curl -X POST "$SLACK_CHANNEL_BUILD_NOTIFICATIONS_WEBHOOK" \
            --silent --show-error --header "Content-Type: application/json" --data @-
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
}
