{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
}@args:

let pkgs = import ./nix/nixpkgs.nix args; in {
  inherit pkgs;
} // pkgs.dfinity-sdk.packages
