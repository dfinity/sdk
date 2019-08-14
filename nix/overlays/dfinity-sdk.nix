self: super:

let dfx = super.callPackage ../../dfx/package.nix {
  libressl = self.libressl_2_9;
}; in

{
  dfinity-sdk = {
    inherit dfx;

    # This is to make sure CI evalutes shell derivations, builds their dependencies
    # and populates the hydra cache with them.
    shells = {
      dfx = import ../../dfx/shell.nix { pkgs = self; };
    };

    licenses = {
      dfx = super.lib.runtime.runtimeLicensesReport dfx;
    };
  };
}
