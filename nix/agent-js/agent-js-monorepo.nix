{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
# This should be via sourcesnix for the git monorepo
, agent-js-monorepo-src
}:
let
  src = agent-js-monorepo-src;
in
pkgs.stdenv.mkDerivation {
  name = "agent-js-monorepo";
  src = src;
  buildInputs = [ pkgs.nodejs ];
  outputs = [
    "out"
    "lib"
  ];
  buildPhase = ''
    mkdir -p .npm-cache
    # without this, npm install will try to write to ~/.npm, which isn't writable in nix
    export NPM_CONFIG_CACHE=.npm-cache;
    # npm install;
    npx lerna bootstrap --nohoist '*';
  '';
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    test -d node_modules && cp -R node_modules $lib || true
  '';
}
