self: super:

let dfx = super.callPackage ../../dfx/package.nix {
  inherit (self) actorscript dfinity runCommand;
}; in

let js-user-library = super.callPackage ../../js-user-library/package.nix {
  inherit (self) napalm;
}; in

{
  dfinity-sdk = {
    inherit dfx js-user-library;

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      dfx = import ../../dfx/shell.nix { pkgs = self; };
      js-user-library = import ../../js-user-library/shell.nix { pkgs = self; };
    };

    licenses = {
      dfx = super.lib.runtime.runtimeLicensesReport dfx;
      # FIXME js-user-library
    };
  };
}
