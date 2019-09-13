# A nixpkgs overlay adding the top-level tools/packages
self: super:
rec {
  rustc = self.rustPackages.rustc;
  cargo = self.rustPackages.cargo;

  # These are used by various targets
  libressl = super.libressl_2_9;

  # rustfmt needs to be compatible with rustc. The rust repo actually specifies
  # the compatible rustfmt src using a git submodule.
  # self.pkgsUnstable.rustfmt is a derivation for rustfmt based on that src.
  # See: https://github.com/NixOS/nixpkgs/pull/66713
  rustfmt = super.pkgsUnstable.rustfmt;
  rls = super.pkgsUnstable.rls;
}
