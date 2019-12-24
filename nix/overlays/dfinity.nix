self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "b3b4acfe101da67b6b6cbeed7d37895b446f3020";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
