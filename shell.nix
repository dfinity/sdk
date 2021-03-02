{ pkgs ? import ./nix {} }:
let
  packages = import ./. { inherit pkgs; };
  nixFmt = pkgs.lib.nixFmt {
    excludeSuffix = [ "Cargo.nix" ];
  };
in
pkgs.mkCompositeShell {
  name = "dfinity-sdk-env";
  inputsFrom = pkgs.stdenv.lib.attrValues packages.shells;
  nativeBuildInputs = [ nixFmt.fmt ];
}
