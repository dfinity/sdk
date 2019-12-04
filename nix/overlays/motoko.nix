self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "joachim/ic-calls-on-drun";
  rev = "b2899e3c7b48ce90032739a45c4d61b60ecc5bf8";
}; in

{
  motoko = import src { system = self.system; };
}
