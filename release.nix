{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []

  # Hydra will pass an argument to this jobset expression for every
  # input of the jobset. In our case we only have a single `src` input
  # representing the git checkout of the SDK repo:
  #
  #   https://hydra.dfinity.systems/jobset/dfinity-ci-build/sdk#tabs-configuration
  #
  # This `src` argument contains information about the input for example:
  #
  #   { outPath = builtins.storePath /nix/store/ma2dfyfyxdi6idbza6dyp34zhxh12nmm-source;
  #     inputType = "git";
  #     uri = "git@github.com:dfinity-lab/sdk.git";
  #     rev = "8882d8c97decbb3c33923ca9e8ab785621a61207";
  #     revCount = 2514;
  #     gitTag = "8882d8c9"; # This defaults to the rev when there's no git tag.
  #     shortRev = "8882d8c9";
  #   }
  #
  # See: https://github.com/NixOS/hydra/blob/037f6488f633dbf15ca2d93a0c117f6738c707fe/src/lib/Hydra/Plugin/GitInput.pm#L241:L248
  #
  # We're  primarily interested in the `src.gitTag` since that should trigger a release.
, src ? null
}:
let
  # doRelease is true when the git tag is of the right release format like `0.1.2`.
  doRelease = src != null && versionMatches != null;

  # versionMatch is `null` if `src.gitTag` is not of the right format like "1.23.456"
  # and it's a list of matches like [ "1.23.456" ] when it is.
  versionMatches = builtins.match "([0-9]+\.[0-9]+\.[0-9]+)" src.gitTag;
  releaseVersion = if versionMatches == null then "unreleased" else builtins.head versionMatches;

  pkgs = import ./nix { inherit system config overlays releaseVersion; };

  packages = import ./. { inherit pkgs src; };

in
pkgs.lib.optionalAttrs doRelease {
  inherit (packages) dfx-release install-sh-release;
}
