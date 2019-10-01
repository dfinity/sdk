self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "c14c1d8013d35d1894f6f186b1802616f46983de";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
