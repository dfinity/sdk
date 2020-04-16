{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, userlib-js ? import ../../src/userlib/js { inherit pkgs; }
}:
pkgs.napalm.buildPackage (pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../../. ./.)) {
  root = ./.;
  name = "node-e2e-tests";
  buildInputs = [
    dfx.standalone
    # Required by node-gyp
    pkgs.python3
  ] ++ pkgs.lib.optional pkgs.stdenv.isDarwin
    # Required by fsevents
    pkgs.darwin.apple_sdk.frameworks.CoreServices;

  npmCommands = [
    "npm install"

    # Monkey-patch the userlib source into our install dir. napalm is unable
    # to include dependencies from package-locks in places other than the
    # build root.
    (
      pkgs.writeScript "include-userlib.sh" ''
        #!${pkgs.stdenv.shell}
        set -eo pipefail

        userlib="node_modules/@internet-computer/userlib"
        mkdir -p $userlib

        tar xvzf ${userlib-js.out}/internet-computer-*.tgz --strip-component 1 --directory $userlib/
        cp -R ${userlib-js.lib}/node_modules .
      ''
    )
    "npm run ci"
  ];

  installPhase = ''
    echo Done.
    touch $out
  '';
}
