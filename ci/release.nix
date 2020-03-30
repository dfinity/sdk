{ src ? null }:
let
  # doRelease is true when the git tag is of the right release format like `0.1.2`.
  doRelease = src != null && versionMatches != null;

  # versionMatch is `null` if `src.gitTag` is not of the right format like "1.23.456"
  # and it's a list of matches like [ "1.23.456" ] when it is.
  versionMatches = builtins.match "([0-9]+\.[0-9]+\.[0-9]+)" src.gitTag;
  releaseVersion = if versionMatches == null then "latest" else builtins.head versionMatches;

  ci = import ./ci.nix { inherit src releaseVersion; };
in
if !doRelease then {} else {
  # TODO: remove these jobs when the `publish.dfx` job below
  # is working successfully and the CloudFront CDN is online.
  # See: https://dfinity.atlassian.net/browse/INF-1143
  inherit (ci) dfx-release install-sh-release;

  publish.dfx.x86_64-linux = ci.publish.dfx.x86_64-linux;
}
