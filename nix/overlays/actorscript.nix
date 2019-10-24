self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "a5f8c29d4f17553655fe75fe4fe1092fad746908";
}; in

{
  actorscript = (import src { nixpkgs = self; });
}
