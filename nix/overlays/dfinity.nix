self: super:

let src = builtins.fetchGit {
  name = "dfinity-sources";
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  rev = "86f4f3343c8b4e9c54c0f8542b9f63a48359c866";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
