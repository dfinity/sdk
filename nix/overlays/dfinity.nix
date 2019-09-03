self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "e7f6f38dac9dc179ad3266679b22ed803a3672e4";
}; in

{
  dfinity = (import src {}).dfinity;
}
