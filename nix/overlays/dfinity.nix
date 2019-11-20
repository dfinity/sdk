self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "master";
  rev = "5c7efff0524adbf97d85b27adb180e6137a3428f";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
