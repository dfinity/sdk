self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "joachim/nix-system";
  rev = "761fdee48f3772f341bd5947094c64e587b35ed4";
}; in

{
  motoko = import src { system = self.system; };
}
