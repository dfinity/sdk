{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
}: {
  inherit (import ./nix { inherit system crossSystem config overlays; }) dfinity-sdk;
}
