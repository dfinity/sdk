self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "93f86a53d6541b88b9427ae32fe492368b403afe";
}; in

{
  dfinity = (import src {}).dfinity;
}
