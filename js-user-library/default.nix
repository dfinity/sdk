{ system ? builtins.currentSystem }:
(import ../. { inherit system; }).dfinity-sdk.js-user-library
