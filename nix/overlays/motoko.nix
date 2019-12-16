self: super:

let src = builtins.fetchGit {
  name = "motoko-sources";
  url = "ssh://git@github.com/dfinity-lab/motoko";
  ref = "master";
  rev = "75e091917063f6607600343c4eb79cacb38aa53e";
}; in

{
  motoko = import src { system = self.system; };
}
