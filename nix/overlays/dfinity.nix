self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "af802ab2d5758522525dcdc4c24a0fd95a950449";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
