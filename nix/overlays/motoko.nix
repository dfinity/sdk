self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "62f6e414583d0c7efa1bce04690f4bbc4d789fba";
}; in

{
  motoko = import src { system = self.system; };
}
