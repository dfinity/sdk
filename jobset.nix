{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
}: {
  inherit ((import ./. { inherit system crossSystem config; }).pkgs.dfinity-sdk) packages;
}
