self: super:

let dfx = super.callPackage ../../dfx/package.nix {
  inherit (self) actorscript dfinity;
  libressl = self.libressl_2_9;

  # rustfmt needs to be compatible with rustc. The rust repo actually
  # specifies the compatible rustfmt src using a git submodule.
  # self.pkgsUnstable.rustfmt is a derivation for rustfmt based on that
  # src. See: https://github.com/NixOS/nixpkgs/pull/66713
  rustfmt = self.pkgsUnstable.rustfmt;
  rls = self.pkgsUnstable.rls;
}; in
let dfx-mock = super.callPackage ../../dfx-mock/package.nix {
  libressl = self.libressl_2_9;

  # rustfmt needs to be compatible with rustc. The rust repo actually
  # specifies the compatible rustfmt src using a git submodule.
  # self.pkgsUnstable.rustfmt is a derivation for rustfmt based on that
  # src. See: https://github.com/NixOS/nixpkgs/pull/66713
  rustfmt = self.pkgsUnstable.rustfmt;
  rls = self.pkgsUnstable.rls;
}; in

{
  dfinity-sdk = {
    inherit dfx dfx-mock;

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      dfx = import ../../dfx/shell.nix { pkgs = self; };
      dfx-mock = import ../../dfx-mock/shell.nix { pkgs = self; };
    };

    licenses = {
      dfx = super.lib.runtime.runtimeLicensesReport dfx;
      dfx-mock = super.lib.runtime.runtimeLicensesReport dfx;
    };
  };
}
