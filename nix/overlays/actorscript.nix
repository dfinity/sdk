self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/actorscript";
  ref = "master";
  rev = "6663c7b54c479aa6fd2df69edd8cd5f0afb04a23";
}; in

{
  actorscript = (import src {});
}
