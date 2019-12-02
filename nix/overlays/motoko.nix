self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "213ddb04c7c38db03db17dcbd97bb1d80422b5e8";
}; in

{
  motoko = import src { system = self.system; };
}
