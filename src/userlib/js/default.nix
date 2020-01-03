{ system ? builtins.currentSystem }:
(import ../. { inherit system; }).pkgs.dfinity-sdk.packages.js-user-library
