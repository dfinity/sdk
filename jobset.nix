{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, doRelease ? false
}: {
  inherit (import ./nix { inherit system crossSystem config overlays doRelease; }) dfinity-sdk;
}
