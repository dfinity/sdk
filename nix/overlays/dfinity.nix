self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "d0dc3a032c2dfa6782bcd8925fd9bf4a36b0870d";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
