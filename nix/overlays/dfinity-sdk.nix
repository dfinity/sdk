self: super:
let
  mkRelease = super.callPackage ./mk-release.nix {};
  rust-package' = import ../../dfx.nix { pkgs = self; };
  # remove some stuff leftover by callPackage
  rust-package = removeAttrs rust-package'
    [ "override" "overrideDerivation" ];
  rust-workspace = rust-package.build;
  public = import ../../public { pkgs = self; };
in {
  dfinity-sdk = rec {
    packages =
      # remove the shell since it's being built below in "shells"
      removeAttrs rust-package [ "shell" ] // rec {
        inherit rust-workspace;
        rust-workspace-debug = rust-package.debug;

        userlib.js = import ../../src/userlib/js { pkgs = self; };

        rust-workspace-standalone = super.lib.standaloneRust
          { drv = rust-workspace;
            exename = "dfx";
          };

        e2e-tests = super.callPackage ../../e2e {};
    } //
    # We only run `cargo audit` on the `master` branch so to not let PRs
    # fail because of an updated RustSec advisory-db. Also we only add the
    # job if the RustSec advisory-db is defined. Note that by default
    # RustSec-advisory-db is undefined (null). However, on Hydra the
    # `sdk` master jobset has RustSec-advisory-db defined as an
    # input. This means that whenever a new security vulnerability is
    # published or when Cargo.lock has been changed `cargo audit` will
    # run.
    self.lib.optionalAttrs (self.isMaster && self.RustSec-advisory-db != null) {
      cargo-security-audit = self.lib.cargo-security-audit {
        name = "dfinity-sdk";
        cargoLock = ../../Cargo.lock;
        db = self.RustSec-advisory-db;
        ignores = [];
      };
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
  };
}
