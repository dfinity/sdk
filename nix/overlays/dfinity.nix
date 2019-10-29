self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "50626f51b377c6cdf955b2b41dfae9e550025361";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
