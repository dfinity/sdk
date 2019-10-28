self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "b6d5e10dabb2665a7468809ecf49d1952b79a84a";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
