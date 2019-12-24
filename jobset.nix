{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, RustSec-advisory-db ? null
}: {
  inherit (import ./nix {
    inherit system crossSystem config overlays RustSec-advisory-db;
  }) dfinity-sdk;
}
