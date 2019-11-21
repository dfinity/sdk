self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "fe634f3d48b72e24cd9f46ae2e316dfbc99d9bdc";
}; in

{
  motoko = import src { };
}
