# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, releaseVersion ? "latest"
, RustSec-advisory-db ? null
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
      rev = "7fc0d40dc6e52a221b370be5a04292e59b32f17d";
    };
in import commonSrc {
  inherit system crossSystem config;
  overlays = import ./overlays ++ [
    (_self: _super: {
      inherit
        releaseVersion
        # The dfinity-sdk.packages.cargo-security-audit job has this RustSec
        # advisory-db as a dependency so we add it here to the package set so
        # that job has access to it.
        RustSec-advisory-db;
    })
  ] ++ overlays;
 }
