{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
}: {
  inherit (import ./. { inherit system crossSystem config; }) dfinity-sdk;
}
