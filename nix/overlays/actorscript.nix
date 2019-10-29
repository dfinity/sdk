self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "d931ee5b45d7d1e0d1e73bda5d0a82c874499969";
}; in

let actorscript = import src { nixpkgs = self; }; in

{
  actorscript = actorscript // {
    stdlib = actorscript.stdlib.overrideAttrs (oldAttrs: {
      installPhase = ''
        mkdir -p $out
        cp ${src}/stdlib/*.as $out
        rm $out/*Test.as
      '';
    });
  };
}
