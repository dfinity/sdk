{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
, sources ? import ../sources.nix { inherit system; }
}:
let
  src = sources.agent-js-monorepo;
in

pkgs.stdenv.mkDerivation {
  name = "agent-js-monorepo";
  src = src;
  outputs = [
    "out"
    "lib"
  ];
  buildPhase = ''
    # # npm is not found :(
    # npm install;
  '';
  installPhase = ''
    mkdir -p $out

    cp -R ./* $out/

    # Copy node_modules to be reused elsewhere.
    mkdir -p $lib
    # cp -R node_modules $lib
  '';
}

# # This does not work. napalm doesn't like how `npm install` triggers lerna bootstrap.
# pkgs.napalm.buildPackage src {
#   name = "agent-js-monorepo";
#   outputs = [ "out" "lib" ];
#   # ci script now does everything CI should do. Bundle is needed because it's the output
#   # of the nix derivation.
#   npmCommands = [
#     "npm install"
#   ];
#   installPhase = ''
#     mkdir -p $out

#     cp -R ./* $out/

#     # Copy node_modules to be reused elsewhere.
#     mkdir -p $lib
#     cp -R node_modules $lib
#   '';
# }
