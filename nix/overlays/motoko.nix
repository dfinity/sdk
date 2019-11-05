self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "a9eaa4e72a8279544e0ab813688b5ad677983bdd";
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
