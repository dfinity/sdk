self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "a82dd5ae3ac2aca5e2f5b91303c6efb2ba913613";
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
