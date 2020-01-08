[
  (_self: super: { isMaster = super.isMaster or false; })
  (import ./sources.nix)
  (self: _:
    # some dependencies
    { motoko = import self.sources.motoko { system = self.system; };
      dfinity = (import self.sources.dfinity { inherit (self) system; }).dfinity.rs;
      napalm = self.callPackage self.sources.napalm {
        pkgs = self // { nodejs = self.nodejs-12_x; };
      };
    }
  )
  (import ./dfinity-sdk.nix)
]
