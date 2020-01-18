[
  (_self: super: { isMaster = super.isMaster or false; })
  (import ./sources.nix)
  (
    self: _:
      let
        nixFmt = self.lib.nixFmt { root = ../../.; };
      in
        # some dependencies
        {
          motoko = import self.sources.motoko { system = self.system; };
          dfinity = (import self.sources.dfinity { inherit (self) system; }).dfinity.rs;
          napalm = self.callPackage self.sources.napalm {
            pkgs = self // { nodejs = self.nodejs-12_x; };
          };

          inherit (nixFmt) nix-fmt;
          nix-fmt-check = nixFmt.check;
        }
  )
  (import ./dfinity-sdk.nix)
]
