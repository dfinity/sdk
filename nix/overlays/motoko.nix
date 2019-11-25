self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "2f71cfc9590741425db752a029e0758f94284e79";
}; in

{
  motoko = import src { system = self.system; };
}
