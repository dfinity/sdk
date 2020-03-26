{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx ? import ../../dfx.nix { inherit pkgs; }
, userlib-js ? import ../../src/userlib/js { inherit pkgs; }
}:
let
  e2e = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../../. ./.);
  inputs = with pkgs; [
    coreutils
    dfx.standalone
    nodejs-12_x
  ];
in

pkgs.napalm.buildPackage e2e {
  root = ./.;
  name = "node-e2e-tests";
  buildInputs = inputs;
  PATH = pkgs.lib.makeSearchPath "bin" inputs;

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
