{ pkgs ? import ./nix {} }:
let
  packages = import ./. { inherit pkgs; };
in
pkgs.mkCompositeShell {
  name = "dfinity-sdk-env";
  inputsFrom = pkgs.stdenv.lib.attrValues packages.shells;
  nativeBuildInputs = [ pkgs.nix-fmt ];
}
