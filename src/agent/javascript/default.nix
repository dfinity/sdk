{ pkgs ? import ../../../nix { inherit system; }
, system ? builtins.currentSystem
}:
let
  src = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ./.);
in
pkgs.napalm.buildPackage src {
  root = ./.;
  name = "dfinity-sdk-agent-js";

  outputs = [ "out" "lib" ];

  propagatedNativeBuildInputs = [
    # Required by node-gyp
    pkgs.python3
  ];
  propagatedBuildInputs = pkgs.lib.optional pkgs.stdenv.isDarwin
    # Required by fsevents
    pkgs.darwin.apple_sdk.frameworks.CoreServices;

  # ci script now does everything CI should do. Bundle is needed because it's the output
  # of the nix derivation.
  npmCommands = [
    "npm install"
    "npm run ci"
    "npm run bundle"
  ];

  installPhase = ''
    npm pack
    mkdir -p $out

    cp dfinity-*.tgz $out

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    cp -R node_modules $lib
  '';
}
