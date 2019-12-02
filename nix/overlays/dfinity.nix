self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "997fdc0968ea737e9c2853d6ed4a29279eb9a57a";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
