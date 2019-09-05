self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "93dc597cf017ebf24ced2a0bfd667520ba8a4b5a";
}; in

{
  dfinity = (import src {}).dfinity;
}
