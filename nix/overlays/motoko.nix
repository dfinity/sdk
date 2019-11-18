self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "c1d1a41c52437fe2fce2d4ce8a9ebe19c83a39e5";
}; in

let motoko = import src { }; in

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
