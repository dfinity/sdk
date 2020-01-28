self: super:
let
  rust-package = import ../../dfx.nix { pkgs = self; };
  rust-workspace = rust-package.build;
in
{
  lib = super.lib // { mkRelease = super.callPackage ./mk-release.nix {} ; };
  dfinity-sdk = rec {
    inherit rust-package;
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
  };
}
