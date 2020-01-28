self: super: {
  lib = super.lib // { mkRelease = super.callPackage ./mk-release.nix {}; };
}
