self: super:
let
  mkRelease = super.callPackage ./mk-release.nix {};

in {
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

    dfx-release = mkRelease "dfx"
      # This is not the tagged version, but something afterwards
      "latest" # once INF-495 is in, we will use: packages.rust-workspace.version
      packages.rust-workspace-standalone
      "dfx";

    # The following prepares a manifest for copying install.sh
    # TODO: streamline mkRelease and this
    install-sh-release =
      let version = "latest";
      in self.lib.linuxOnly (super.runCommandNoCC "install-sh-release" {
        inherit version;
        installSh = ../../public/install.sh;
        buildInputs = [ self.jo ];
      } ''
        set -Eeuo pipefail
        # Building the artifacts
        mkdir -p $out

        # Creating the manifest
        manifest_file=$out/manifest.json

        sha256hash=($(sha256sum "$installSh")) # using this to autosplit on space
        sha1hash=($(sha1sum "$installSh")) # using this to autosplit on space

        jo -pa \
          $(jo package="public" \
              version="$version" \
              name="installer" \
              file="$installSh" \
              sha256hash="$sha256hash" \
              sha1hash="$sha1hash") >$manifest_file

        # Marking the manifest for publishing
        mkdir -p $out/nix-support
        echo "upload manifest $manifest_file" >> \
          $out/nix-support/hydra-build-products
      '');

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
