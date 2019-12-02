self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "a77f9b30fa5b1f35bef2913c2329e2c8e81c1af8";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
