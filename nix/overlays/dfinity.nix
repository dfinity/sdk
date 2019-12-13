self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "21f7cb12a737218b952a82f03066308d833e51d9";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
