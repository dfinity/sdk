self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "8639cf5c643c6a9a8998ba831edc7836f357c571";
}; in

{
  actorscript = (import src {});
}
