self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "joachim/nix-system";
  rev = "195849eeae1dba654133aad79e6d2464493f4bf1";
}; in

{
  motoko = import src { system = self.system; };
}
