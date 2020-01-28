{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
, src ? null
, RustSec-advisory-db ? null
}@args: {
  inherit (import ./nix args) dfinity-sdk ;
}
