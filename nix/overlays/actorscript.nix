self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "62a3e50336ac3d99960a9b75e83fc8f972066909";
}; in

{
  actorscript = (import src { nixpkgs = self; });
}
