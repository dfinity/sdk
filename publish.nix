# Build a script to copy the dfx and install.sh release to the
# dfinity-download S3 bucket which is the origin of the
# download.dfinity.systems world-wide CDN.
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

  s3cpHashed = pkgs.lib.writeCheckedShellScriptBin "s3cpHashed" [] ''
    set -eu
    PATH="${pkgs.lib.makeBinPath [ s3cp pkgs.awscli ]}"

    file="$1";
    dir="$2";
    name="$3";
    contentType="$4";
    cacheControlFile="$5"; # cache header of the content
    cacheControlRedirect="$6" # cache header of the redirect

    # Hash (SHA-256) the content and copy it to S3 under a content-addressable name.
    # It's copied to a directory indexed by $name such that we can remove old entries later on.
    hash="$(sha256sum "$file" | cut -d' ' -f1)"
    dstDir="$dir/hashed/$name"
    dst="$dstDir/$hash"
    s3cp "$file" "$dst" "$contentType" "$cacheControlFile"

    # Install a redirect from the fixed name $dir/$name to the content addressable name $dst.
    # The redirect will use a different caching header than the content it redirects to.
    # This enables us to for example cache the content for long but not cache the redirect
    # such that we can always update it quickly.
    src="s3://$DFINITY_DOWNLOAD_BUCKET/$dir/$name"
    echo "Redirecting $src to /$dst..."
    aws s3 cp --website-redirect "/$dst" "$src" \
      --cache-control "$cacheControlRedirect" \
      --no-progress

    echo "Removing old versions in s3://$DFINITY_DOWNLOAD_BUCKET/$dstDir..."
    aws s3 rm "s3://$DFINITY_DOWNLOAD_BUCKET/$dstDir" --recursive --exclude "$dst" --only-show-errors
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

  v = releaseVersion;

  mkDfxTarball = dfx:
    pkgs.runCommandNoCC "dfx-${v}.tar.gz" {
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
  dfx = pkgs.lib.linuxOnly (
    pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
      set -eu
      PATH="${pkgs.lib.makeBinPath [ s3cp slack ]}"

      v="${v}"
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
    ''
  );
  install-sh = pkgs.lib.linuxOnly (
    pkgs.lib.writeCheckedShellScriptBin "activate" [] ''
      set -eu
      PATH="${pkgs.lib.makeBinPath [ s3cpHashed ]}"

      cache_month="max-age=2419200"
      do_not_cache="max-age=0,no-cache"

      s3cpHashed "${./public/manifest.json}" "sdk" "manifest.json" "application/json" "$cache_month" "$do_not_cache"
      s3cpHashed "${install.x86_64-linux}" "sdk" "install.sh" "application/x-sh" "$cache_month" "$do_not_cache"
    ''
  );
}
