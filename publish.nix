# Build a script to copy the dfx and install.sh release to the
# dfinity-download S3 bucket which is the origin of the
# download.dfinity.systems world-wide CDN.
#
# This script will be executed by DFINITY's Continuous Deployment
# system. That system will also set the correct AWS credentials and the
# DFINITY_DOWNLOAD_BUCKET environment variable.
{ pkgs, releaseVersion, dfx-release, install-sh-release }:
let
  s3cp = pkgs.lib.writeCheckedShellScriptBin "s3cp" [] ''
    set -eu
    PATH="${pkgs.lib.makeBinPath [ pkgs.awscli ]}"
    pkg="$1"; path="$2"; dstDir="$3"; contentType="$4"; cacheControl="$5"
    src="$pkg/$path"
    dst="s3://$DFINITY_DOWNLOAD_BUCKET/$dstDir/$path"
    if [ -d "$src" ]; then
      echo "Can't copy $src to $dst because it's a directory. Please specify a file instead." 1>&2; exit 1;
    fi
    echo "Uploading $src to $dst (--cache-control $cacheControl, --content-type $contentType)..."
    aws s3 cp "$src" "$dst" \
      --cache-control "$cacheControl" \
      --no-guess-mime-type --content-type "$contentType" \
      --no-progress
  '';

  slack = pkgs.lib.writeCheckedShellScriptBin "slack" [] ''
    set -eu
    PATH="${pkgs.lib.makeBinPath [ pkgs.jq pkgs.curl ]}"
    slack_channel_webhook="$1"
    msg="$(</dev/stdin)"
    echo {} | jq --arg msg "$msg" '.blocks=[
      {
        "type" : "section",
        "text" : {
          "type" : "mrkdwn",
          "text" : $msg
        }
      }
    ]' | curl -X POST --data @- "$slack_channel_webhook" \
           --header "Content-Type: application/json" --silent --show-error
  '';
in
{
  dfx = pkgs.lib.linuxOnly (
    pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
      set -eu
      PATH="${pkgs.lib.makeBinPath [ s3cp slack ]}"

      v="${releaseVersion}"
      cache_long="max-age=31536000" # 1 year

      path="dfx-$v.tar.gz"
      dir="sdk/dfx/$v"

      s3cp "${dfx-release.x86_64-linux}" "$path" "$dir/x86_64-linux" "application/gzip" "$cache_long"
      s3cp "${dfx-release.x86_64-darwin}" "$path" "$dir/x86_64-darwin" "application/gzip" "$cache_long"

      slack "$SLACK_CHANNEL_BUILD_NOTIFICATIONS_WEBHOOK" <<EOI
      *DFX-$v* has been published to DFINITY's CDN :champagne:!
      - https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-linux/$path
      - https://$DFINITY_DOWNLOAD_DOMAIN/$dir/x86_64-darwin/$path
      Install the SDK by following the instructions at: https://sdk.dfinity.org/docs/download.html.
      EOI
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

      pkg="${install-sh-release.x86_64-linux}"
      s3cp "$pkg" "manifest.json" "sdk" "application/json" "$do_not_cache"
      s3cp "$pkg" "install.sh" "sdk" "application/x-sh" "$do_not_cache"
    ''
  );
}
