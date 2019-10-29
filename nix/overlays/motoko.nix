self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "d931ee5b45d7d1e0d1e73bda5d0a82c874499969";
}; in

let motoko = import src { nixpkgs = self; }; in

{
  motoko = motoko // {
    stdlib = motoko.stdlib.overrideAttrs (oldAttrs: {
      installPhase = ''
        mkdir -p $out
        cp ${src}/stdlib/*.mo $out
        rm $out/*Test.mo
      '';
    });
  };
}
