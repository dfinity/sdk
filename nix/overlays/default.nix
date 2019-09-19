[
  (import ./sources.nix)
  (import ./mkShell)
  (import ./rust.nix)
  (import ./naersk.nix)
  (import ./licenses.nix)
  (import ./lib)
  (import ./packages)
  (import ./actorscript.nix)
  (import ./dfinity.nix)
  (import ./dfinity-sdk.nix)
  (import ./mkCiShell.nix)
  # This file must be the last mentioned as it uses outputs from most other files
  # and lifts them to the top-level package set
  (import ./top-level.nix)
]
