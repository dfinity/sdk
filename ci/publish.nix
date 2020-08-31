# Build a script to copy the dfx and install.sh release to the
# dfinity-download S3 bucket which is the origin of the
# download.dfinity.systems world-wide CDN.
#
# To run the dfx deployment script locally:
#   * nix run -f ./ci/ci.nix publish.dfx -c activate
# and make sure that the environment variables
# "SLACK_CHANNEL_BUILD_NOTIFICATIONS_WEBHOOK" and "DFINITY_DOWNLOAD_DOMAIN" are
# set and that the environment variable "DFINITY_DOWNLOAD_BUCKET" is set and
# that you have access to the bucket.
#
# To run the install deployment script locally:
#   * nix run -f ./ci/ci.nix publish.install-sh -c activate
#  and make sure your AWS credentials are set, that the environment variable
#  "DFINITY_DOWNLOAD_BUCKET" is set and that you have access to the bucket.
#
# This script will be executed by DFINITY's Continuous Deployment
# system. That system will also set the correct AWS credentials and the
# DFINITY_DOWNLOAD_BUCKET environment variable.
{ pkgs, releaseVersion, dfx, install }:
let
  s3cp = pkgs.lib.writeCheckedShellScriptBin "s3cp" [] ''
    set -eu
    PATH="${pkgs.lib.makeBinPath [ pkgs.awscli ]}"
    src="$1"; dst="$2"; contentType="$3"; cacheControl="$4"
    dstUrl="s3://$DFINITY_DOWNLOAD_BUCKET/$dst"
    if [ -d "$src" ]; then
      echo "Can't copy $src to $dstUrl because it's a directory. Please specify a file instead." 1>&2; exit 1;
    fi
    echo "Uploading $src to $dstUrl (--cache-control $cacheControl, --content-type $contentType)..."
    aws s3 cp "$src" "$dstUrl" \
      --cache-control "$cacheControl" \
      --no-guess-mime-type --content-type "$contentType" \
      --no-progress
  '';

  mkDfxTarball = dfx:
    pkgs.runCommandNoCC "dfx-${releaseVersion}.tar.gz" {
      inherit dfx;
      allowedRequisites = [];
    } ''
      tmp=$(mktemp -d)
      cp $dfx/bin/dfx $tmp/dfx
      chmod 0755 $tmp/dfx
      tar -czf "$out" -C $tmp/ .
    '';

in
{
  dfx = pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
    set -eu
    PATH="${pkgs.lib.makeBinPath [ s3cp pkgs.slack ]}"

    v="${releaseVersion}"
    cache_long="max-age=31536000" # 1 year

    file="dfx-$v.tar.gz"
    dir="sdk/dfx/$v"

    s3cp "${mkDfxTarball dfx.x86_64-linux}" "$dir/x86_64-linux/$file" "application/gzip" "$cache_long"
    s3cp "${mkDfxTarball dfx.x86_64-darwin}" "$dir/x86_64-darwin/$file" "application/gzip" "$cache_long"

    slack "$SLACK_CHANNEL_BUILD_NOTIFICATIONS_WEBHOOK" <<EOI
    *DFX-$v* has been published to DFINITY's CDN :champagne:!
    - https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-linux/$file
    - https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-darwin/$file
    Install the SDK by following the instructions at: https://sdk.dfinity.org/docs/download.html.
    EOI
  '';
  install-sh = pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
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

    s3cp "${../public/manifest.json}" "sdk/manifest.json" "application/json" "$do_not_cache"
    s3cp "${../public/sdk-license-agreement/index.txt}" "sdk/sdk-license-agreement.txt" "application/txt" "$do_not_cache"
    s3cp "${../public/tungsten-license-agreement.txt}" "sdk/tungsten-license-agreement.txt" "application/txt" "$do_not_cache"
    s3cp "${install.x86_64-linux}" "sdk/install.sh" "application/x-sh" "$do_not_cache"
  '';
}
