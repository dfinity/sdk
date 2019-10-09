self: super: {
  dfinity-sdk = rec {
    packages = rec {
        js-user-library = super.callPackage ../../js-user-library/package.nix {
          inherit (self) napalm;
        };
        rust-workspace = super.callPackage ../rust-workspace.nix {};
        rust-workspace-debug = (rust-workspace.override (_: {
          release = false;
          doClippy = true;
          doFmt = true;
          doDoc = true;
        })).overrideAttrs (oldAttrs: {
          name = "${oldAttrs.name}-debug";
        });
        rust-workspace-doc = rust-workspace-debug.doc;

        rust-workspace-standalone = (super.lib.standaloneRust rust-workspace "dfx");

        e2e-tests = super.callPackage ../e2e-tests.nix {};

        public-folder = super.callPackage ../public.nix {};
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
      rust-workspace = super.lib.runtime.runtimeLicensesReport packages.rust-workspace;
    };
  };
}
