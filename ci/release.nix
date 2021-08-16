{ src ? null }:
let
  # doRelease is true when the git tag is of the right release format like `0.1.2`.
  doRelease = src != null && versionMatches != null;

  # versionMatch is `null` if `src.gitTag` is not of the right format like "1.23.456"
  # and it's a list of matches like [ "1.23.456" ] when it is.
  versionMatches = builtins.match "([0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?)" src.gitTag;
  releaseVersion = if versionMatches == null then "latest" else builtins.head versionMatches;

  shortRev = src.shortRev or "unknown";

  ci = import ./ci.nix { inherit src releaseVersion; };
in
if src == null then {}
else if !doRelease then builtins.trace ''
  notice: treating this as a non-release commit
  the tag ${src.gitTag} (rev ${shortRev}) does not appear to be a release version
'' {} else {
  publish.dfx = ci.publish.dfx;
}
