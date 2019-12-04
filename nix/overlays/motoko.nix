self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "joachim/ic-calls-on-drun";
  rev = "881ec6855c3859eb45ed97625ac2062fdc9cafd5";
}; in

{
  motoko = import src { system = self.system; };
}
