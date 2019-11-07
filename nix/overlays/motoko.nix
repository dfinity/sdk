self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "b5573dfceaf963256d244eb1e9f5468415f1678b";
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
