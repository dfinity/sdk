self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "852e31c44742c31f77dd759a4888f3b727631f6f";
}; in

{
  actorscript = (import src {});
}
