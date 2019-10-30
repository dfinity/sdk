self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "nm-recover";
  rev = "df7308d4b38b0f8044cb4afda7e0e11fbce7806a";
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
