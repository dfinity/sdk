self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "roman/DFN-1191-call-context";
  rev = "7c632a52054eaa9a8dd2f98aed43d3c4c6fdbac9";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
