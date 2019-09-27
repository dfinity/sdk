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
  # To conveniently test changes to a local `common` repo you point
  # the `COMMON` environment variable to it. The path should be
  # relative to the root of the `sdk` repo. For example:
  #
  #   COMMON="../common" nix-build -A rust-workspace
  commonSrc =
    let localCommonSrc = builtins.getEnv "COMMON"; in
    if localCommonSrc != ""
    then ../. + "/${localCommonSrc}"
    else builtins.fetchGit {
      url = "ssh://git@github.com/dfinity-lab/common";
      rev = "28681f2d12617b69f875d2743103d518136376be";
    };
in import commonSrc {
  inherit system crossSystem config;
  overlays = import ./overlays ++ overlays;
 }
