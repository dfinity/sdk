{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
}@args:

let pkgs = import ./nix {inherit system crossSystem config overlays; }; in {
  inherit pkgs;
} // pkgs.dfinity-sdk.packages
