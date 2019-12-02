self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "24b459bb7497fdff42e836e6a06e3d28f0425f55";
}; in

{
  motoko = import src { system = self.system; };
}
