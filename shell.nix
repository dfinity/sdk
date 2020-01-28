{ pkgs ? import ./nix {} }:
let
  default = import ./. { inherit pkgs; };
in
pkgs.mkCompositeShell {
  name = "dfinity-sdk-env";
  inputsFrom = pkgs.stdenv.lib.attrValues default.shells;
  buildInputs = [ pkgs.nix-fmt ];
}
