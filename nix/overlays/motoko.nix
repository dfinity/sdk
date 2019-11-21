self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "6901c536deff066671aa96c6e765d96a7415bb40";
}; in

{
  motoko = import src { };
}
