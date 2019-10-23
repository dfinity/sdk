self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "bd8b24d7e28638a73e616dabf6077a2869f14f32";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
