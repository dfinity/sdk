{ pkgs ? import ../. { inherit system; }, system ? builtins.currentSystem }:
let
  # derivation that has all system dependencies required to build the npm monorepo:
  # * npm requires python3 to build with gyp
  # * on mac, npm may try to use fsevents
  agentJsMonorepoTools = src:
    pkgs.stdenv.mkDerivation {
      inherit src;
      name = "agent-js-monorepo-systemRequirements";
      propagatedNativeBuildInputs = [
        # Required by node-gyp
        pkgs.python3
      ];
      propagatedBuildInputs = [
        (
          pkgs.lib.optional pkgs.stdenv.isDarwin
            # Required by fsevents
            pkgs.darwin.apple_sdk.frameworks.CoreServices
        )
      ];
      installPhase = ''
        mkdir -p $out
      '';
    };
in
agentJsMonorepoTools
