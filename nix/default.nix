# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
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
      rev = "b4cd9e11b5e36bbdea3cf4674c15c9f689d9f318";
    };
in import commonSrc {
  inherit system crossSystem config;
  overlays = import ./overlays ++ overlays;
 }
