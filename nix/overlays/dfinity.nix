self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "7c6fc6bc06fa0dc4abf0f1b3d6438d39e1e72649";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
