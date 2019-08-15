let pkgs = (import ./. {}).pkgs; in

pkgs.mkShell {
  inputsFrom = pkgs.stdenv.lib.attrValues pkgs.dfinity-sdk.shells;
}
