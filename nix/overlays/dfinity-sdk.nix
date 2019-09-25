self: super:

let rust-workspace = super.callPackage ../rust-workspace.nix {}; in

{
  dfinity-sdk = {
    packages = {
        inherit rust-workspace;
        rust-workspace-debug = rust-workspace.override (_: {
          release = false;
          doClippy = true;
          doFmt = true;
          doDoc = true;
        });
    };

    # This is to make sure CI evalutes shell derivations, builds their
    # dependencies and populates the hydra cache with them. We also use this in
    # `shell.nix` in the root to provide an environment which is the composition
    # of all the shells here.
    shells = {
      rust-workspace = import ../rust-shell.nix { pkgs = self; };
    };

    licenses = {
      rust-workspace = super.lib.runtime.runtimeLicensesReport rust-workspace;
    };
  };
}
