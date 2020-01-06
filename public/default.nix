{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
}:

let src = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../. "public"); in

pkgs.runCommandNoCC "public-folder" {} ''
    mkdir -p $out
    cp -R ${src}/. $out
''
