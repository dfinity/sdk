self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  rev = "5e124292f17f55120738742b93fbbd02e83b14bf";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity.rs;
}
