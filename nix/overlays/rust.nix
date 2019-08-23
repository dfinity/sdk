self: super: {
  pkgsUnstable = import self.sources.nixpkgs-unstable {
    inherit (self) system;
  };
  inherit (self.pkgsUnstable) rustPackages;
}
