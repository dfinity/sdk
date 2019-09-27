{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
}: {
  inherit (import ./nix { inherit system crossSystem config; }) dfinity-sdk;
}
