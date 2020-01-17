{ pkgs ? (import ./. {}).pkgs }:
pkgs.mkCompositeShell {
  name = "dfinity-sdk-env";
  inputsFrom = pkgs.stdenv.lib.attrValues pkgs.dfinity-sdk.shells;
  buildInputs = [ pkgs.nix-fmt ];
}
