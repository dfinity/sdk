# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, releaseVersion ? "latest"
}:
let
  # The `common` repo provides code (mostly Nix) that is used in the
  # `infra`, `dfinity` and `sdk` repositories.
  #
  # To conveniently test changes to a local `common` repo you set the `COMMON`
  # environment variable to an absolute path of it. For example:
  #
  #   COMMON="$(realpath ../common)" nix-build -A rust-workspace
  commonSrc =
    let localCommonSrc = builtins.getEnv "COMMON"; in
    if localCommonSrc != ""
    then localCommonSrc
    else builtins.fetchGit {
      name = "common-sources";
      url = "ssh://git@github.com/dfinity-lab/common";
      rev = "b35ba23941c5f3b555eba33c144f2cf9e5036cfb";
    };
in import commonSrc {
  inherit system crossSystem config;
  overlays = import ./overlays ++ [ (_self: _super: { inherit releaseVersion; }) ] ++ overlays;
 }
