self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "f8c94d70ef76e6301aa226b52386cc5dfcc9f24a";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
