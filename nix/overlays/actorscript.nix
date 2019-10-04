self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "71e333aacd83e05d94c5720cb07419bcd0a1858e";
}; in

{
  actorscript = (import src { nixpkgs = self; });
}
