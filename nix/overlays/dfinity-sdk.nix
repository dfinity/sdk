self: super:
let
  mkRelease = super.callPackage ./mk-release.nix {};
  rust-package = import ../../dfx.nix { pkgs = self; };
  rust-workspace = rust-package.build;
  public = import ../../public { pkgs = self; };
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

        e2e-tests = import ../../e2e { pkgs = self; };

        # The cargo audit job for known vulnerabilities. This generally run
        # against the advisory database pinned in sources.json; on Hydra
        # (master) however the latest advisory database is fetched from
        # RustSec/advisory-db. This means that whenever a new security
        # vulnerability is published or when Cargo.lock has been changed `cargo
        # audit` will run.
        cargo-security-audit = import ../../cargo-audit.nix { pkgs = self; };
      };

    dfx-release = mkRelease "dfx" self.releaseVersion packages.rust-workspace-standalone "dfx";

    inherit (public) install-sh install-sh-lint install-sh-release;

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      js-user-library = import ../../src/userlib/js/shell.nix { pkgs = self; };
      rust-workspace = import ../rust-shell.nix { pkgs = self; shell = rust-package.shell; };
    };

    licenses = {
      rust-workspace = super.lib.runtime.runtimeLicensesReport packages.rust-workspace;
    };

    inherit (self) nix-fmt-check;
  };
}
