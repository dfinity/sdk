self: super:
let
  mkRelease = super.callPackage ./mk-release.nix {};
  rust-package = import ../../dfx.nix { pkgs = self; };
  rust-workspace = rust-package.build;
in
{
  dfinity-sdk = rec {
    packages =
      # remove the shell since it's being built below in "shells"
      removeAttrs rust-package [ "shell" ] // rec {
        inherit rust-workspace;
        rust-workspace-debug = rust-package.debug;

        userlib.js = import ../../src/userlib/js { pkgs = self; };

        rust-workspace-standalone = super.lib.standaloneRust
          {
            drv = rust-workspace;
            exename = "dfx";
            usePackager = false;
          };
      };

    dfx-release = mkRelease "dfx" self.releaseVersion packages.rust-workspace-standalone "dfx";

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      js-user-library = import ../../src/userlib/js/shell.nix { pkgs = self; };
      rust-workspace = import ../../dfx-shell.nix { pkgs = self; inherit rust-package; };
    };

    licenses = {
      rust-workspace = super.lib.runtime.runtimeLicensesReport packages.rust-workspace;
    };
  };
}
