self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "71e333aacd83e05d94c5720cb07419bcd0a1858e";
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
