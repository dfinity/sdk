self: super: {
  dfinity-sdk = rec {
    packages = rec {
      js-user-library = super.callPackage ../../js-user-library/package.nix {
        inherit (self) napalm;
      };
      rust-workspace = super.callPackage ../rust-workspace.nix {};
      rust-workspace-debug = rust-workspace.override (_: {
        release = false;
        doClippy = true;
        doFmt = true;
        doDoc = true;
      });
      rust-workspace-doc = rust-workspace-debug.doc;
    };

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      js-user-library = import ../../js-user-library/shell.nix { pkgs = self; };
      rust-workspace = import ../rust-shell.nix { pkgs = self; };
    };

    licenses = {
      # FIXME js-user-library
      rust-workspace = super.lib.runtime.runtimeLicensesReport packages.rust-workspace;
    };
  };
}
