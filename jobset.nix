{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
}: {
  inherit (import ./nix { inherit system crossSystem config overlays; }) dfinity-sdk;
}
