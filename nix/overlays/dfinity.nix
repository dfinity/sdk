self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  ref = "v0.1.0";
  rev = "104e6e9348148e69ea0ec06bcae2a7c1dae4f558";
}; in

{
  dfinity = (import src {}).dfinity;
}
