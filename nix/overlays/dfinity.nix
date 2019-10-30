self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  rev = "ece3f0cde776bf26bb498fe0bec61cbdd0c3ebc7";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
