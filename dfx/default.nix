{ system ? builtins.currentSystem }:
(import ../. { inherit system; }).dfinity-sdk.dfx
