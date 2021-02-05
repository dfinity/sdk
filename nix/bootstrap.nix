{ pkgs ? import ./. { inherit system; }, system ? builtins.currentSystem }:
let dist = pkgs.fetchurl {
  url = "https://registry.npmjs.org/@dfinity/bootstrap/-/bootstrap-0.0.0.tgz";
  sha256 = "156f669sabrfsy04ap166wx6nmkw38b0njfmfhx17g6q1zr45072";
}; in
pkgs.runCommandNoCC "assets-bootstrap" {} ''
  tar xvf ${dist}
  mkdir -p $out
  mv package/dist/* $out
''
