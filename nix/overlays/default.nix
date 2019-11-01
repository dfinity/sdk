[
  (_self: _super: { repoRoot = ../..; })
  (_self: super: { isMaster = super.isMaster or false; })
  (import ./sources.nix)
  (import ./motoko.nix)
  (import ./dfinity.nix)
  (import ./napalm.nix)
  (import ./dfinity-sdk.nix)
]
