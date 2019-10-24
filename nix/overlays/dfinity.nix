self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "b40b62471693f3700d91362d712fccef88255cec";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
