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
    pkg="$1"; path="$2"; dstDir="$3"; contentType="$4"; cacheControl="$5"
    src="$pkg/$path"
    dst=""s3://$DFINITY_DOWNLOAD_BUCKET/$dstDir/$path""
    if [ -d "$src" ]; then
      echo "Can't copy $src to $dst because it's a directory. Please specify a file instead." 1>&2; exit 1;
    fi
    echo "Uploading $src to $dst (--cache-control $cacheControl, --content-type $contentType)..."
    aws s3 cp "$src" "$dst" \
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
      # See: https://dfinity.atlassian.net/browse/INF-1146
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

      path="dfx-$v.tar.gz"
      dir="sdk/dfx/$v"

      s3cp "${packages_x86_64-linux.dfx-release}" "$path" "$dir/x86_64-linux" "application/gzip" "$cache_long"
      s3cp "${packages_x86_64-darwin.dfx-release}" "$path" "$dir/x86_64-darwin" "application/gzip" "$cache_long"

      msg=$(cat <<EOI
      DFX-$v has been published to DFINITY's CDN at:
      * https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-linux/$path
      * https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-darwin/$path
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

      # TODO: publish the manifest.json and install.sh to content
      # addressable paths which can be cached for a long time. Then publish
      # uncached --website-redirects to redirect sdk/manifest.json and
      # sdk/install.sh to their content addressable alternatives. This way the latest
      # manifest.json and install.sh will be available in the CDN and won't have
      # to be fetched from the origin bucket.
      # See: https://dfinity.atlassian.net/browse/INF-1145

      s3cp "${packages_x86_64-linux.install-sh-release}" "manifest.json" "sdk" "application/x-sh" "$do_not_cache"
      s3cp "${packages_x86_64-linux.install-sh-release}" "install.sh"    "sdk" "application/x-sh" "$do_not_cache"
    ''
  );
}
