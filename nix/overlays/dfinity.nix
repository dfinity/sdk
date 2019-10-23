self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "f3b4e4ce1b0baccdb0407566d31f2cd7a128385a";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
