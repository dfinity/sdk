self: super: {
  # ./mkshell.nix is a copy of <nixpkgs/pkgs/build-support/mkshell/default.nix>
  # from nixpkgs master which has support for composing shellHooks which is not
  # yet supported on release-19.03.
  # Also see: https://github.com/NixOS/nixpkgs/pull/63701
  mkShell = super.callPackage ./mkshell.nix {};
}
