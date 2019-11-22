self: super:
let
  mkRelease = super.callPackage ./mk-release.nix {};
  rust-package' = super.callPackage ../rust-workspace.nix {};
  # remove some stuff leftover by callPackage
  rust-package = removeAttrs rust-package'
    [ "override" "overrideDerivation" ];
  rust-workspace = rust-package.build;
in {
  dfinity-sdk = rec {
    packages = rust-package // rec {
        inherit rust-workspace;
        rust-workspace-debug = rust-package.debug;
        js-user-library = super.callPackage ../../js-user-library/package.nix {
          inherit (self) napalm;
        };

        rust-workspace-standalone = super.lib.standaloneRust
          { drv = rust-workspace;
            exename = "dfx";
          };

        e2e-tests = super.callPackage ../e2e-tests.nix {};

        public-folder = super.callPackage ../public.nix {};
    };

    dfx-release = mkRelease "dfx" self.releaseVersion packages.rust-workspace-standalone "dfx";

    # The following prepares a manifest for copying install.sh
    # The release part also checks if the install.sh script is well formatted and has no shellcheck issues.
    # We ignore 'local' warning by shellcheck, because any existing sh implementation supports it.
    # TODO: streamline mkRelease and this
    install-sh-release =
      let
        version = self.releaseVersion;
        shfmtOpts = "-p -i 4 -ci -bn -s";
        shellcheckOpts = "-s sh -S warning";
        # We want to include the last revision of the install script into
        # the released version of the script
        revision = super.lib.fileContents (
          let
            commondir = super.lib.gitDir + "/commondir";
            isWorktree = builtins.pathExists commondir;
            mainGitDir = super.lib.gitDir + "/${super.lib.fileContents commondir}";
            worktree = super.lib.optionalString isWorktree (
              super.lib.dropString (builtins.stringLength (toString mainGitDir))
                (toString super.lib.gitDir));
          in super.runCommandNoCC "install_sh_timestamp" {
            git_dir = builtins.path {
              name = "sdk-git-dir";
              path = if isWorktree
                     then mainGitDir
                     else super.lib.gitDir;
            };
            nativeBuildInputs = [ self.git ];
            preferLocalBuild = true;
            allowSubstitutes = false;
          } ''
            cd $git_dir${worktree}
            git log -n 1 --pretty=format:%h-%cI -- public/install.sh > $out
          ''
        );
      in self.lib.linuxOnly (super.runCommandNoCC "install-sh-release" {
        inherit version;
        inherit (self) isMaster;
        inherit revision;
        installSh = ../../public/install.sh;
        manifest = ../../public/manifest.json;
        buildInputs = [ self.jo self.shfmt self.shellcheck ];
      } ''
        set -Eeuo pipefail
        # Check if we have an sh compatible script
        shckResult="$(shellcheck -Cnever -f gcc ${shellcheckOpts} "$installSh" | grep -v "In POSIX sh, 'local' is undefined." || true)"
        if [ -n "$shckResult" ] ; then
          echo "There are some shellcheck warnings:"
          echo $shckResult
          echo "Please run:"
          echo "shellcheck ${shellcheckOpts} public/install.sh"
          exit 1
        fi

        # Check if the file is properly formatted
        if ! shfmt ${shfmtOpts} -d $installSh; then
          echo "Please run:"
          echo "shfmt ${shfmtOpts} -w public/install.sh"
          exit 1
        fi
        # Building the artifacts
        mkdir -p $out

        version_manifest_file=$out/manifest.json

        cp $manifest $version_manifest_file
        # we stamp the file with the revision
        substitute "$installSh" $out/install.sh \
          --subst-var revision

        # Creating the manifest
        hydra_manifest_file=$out/_manifest.json

        sha256hashinstall=($(sha256sum "$out/install.sh")) # using this to autosplit on space
        sha1hashinstall=($(sha1sum "$out/install.sh")) # using this to autosplit on space


        sha256manifest=($(sha256sum "$version_manifest_file")) # using this to autosplit on space
        sha1manifest=($(sha1sum "$version_manifest_file")) # using this to autosplit on space

        jo -pa \
          $(jo package="public" \
              version="$version" \
              name="installer" \
              file="$out/install.sh" \
              sha256hash="$sha256hashinstall" \
              sha1hash="$sha1hashinstall") \
          $(jo package="public" \
              version="$version" \
              name="manifest.json" \
              file="$version_manifest_file" \
              sha256hash="$sha256manifest" \
              sha1hash="$sha1manifest") >$hydra_manifest_file

        # Marking the manifest for publishing
        mkdir -p $out/nix-support
        echo "upload manifest $hydra_manifest_file" >> \
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
