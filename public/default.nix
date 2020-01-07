{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
}:

let
  src = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../. "public");
  version = pkgs.releaseVersion;
  repoRoot = ../.;
  gitDir = pkgs.lib.gitDir repoRoot;
in

rec {
  # TODO: this is not actually used and should be removed
  public-folder =
    pkgs.runCommandNoCC "public-folder" {} ''
        mkdir -p $out
        cp -R ${src}/. $out
    '';

  install-sh =
    pkgs.runCommandNoCC "install-sh" {
      public = src;
      buildInputs = [ ];
    } ''
      # git describe --abbrev=7 --tags
      mkdir -p $out

      cat $public/install/*.sh > $out/install.sh

      # Get rid of comments that don't start with '##' or '#!'.
      sed -i "
        /#!.*/p
        /##.*/p
        /^ *$/d
        /^ *#/d
        s/ *#.*//
      " $out/install.sh
    '';

  # Check if the install.sh script is well formatted and
  # has no shellcheck issues.  We ignore 'local' warning by shellcheck, because
  # any existing sh implementation supports it.
  # TODO: streamline mkRelease and this
  install-sh-lint =
    let
      shfmtOpts = "-p -i 4 -ci -bn -s";
      shellcheckOpts = "-s sh -S warning";
    in
      pkgs.runCommandNoCC "install-sh-lint" {
        inherit version;
        inherit (pkgs) isMaster;
        public = src;
        buildInputs = [ install-sh pkgs.shfmt pkgs.shellcheck ];
      } ''
        set -Eeuo pipefail
        # Check if we have an sh compatible script
        shckResult="$(shellcheck -Cnever -f gcc ${shellcheckOpts} "$public/install.sh" | \
            grep -v "warning: In POSIX sh, 'local' is undefined. \[SC2039\]" | \
            sed -e "s%^${install-sh}/?%%g" || true)"

        if [ -n "$shckResult" ] ; then
          echo "There are some shellcheck warnings:"
          echo
          echo "$shckResult"
          echo
          exit 1
        fi

        # Check if the file is properly formatted
        if ! shfmt ${shfmtOpts} -d "${install-sh}/install.sh"; then
          echo "Formatting error. Please run:"
          echo
          echo "shfmt ${shfmtOpts} -w public/install.sh"
          exit 1
        fi

        if grep source "${install-sh}/install.sh"; then
          echo "Found a source above in the output. There should be none remaining (inlined)."
          exit 1
        fi

        # Make sure Nix sees the output.
        touch $out
      '';

  # The following prepares a manifest for copying install.sh
  install-sh-release =
    let
      # We want to include the last revision of the install script into
      # the released version of the script
      revision = pkgs.lib.fileContents (
        let
          commondir = gitDir + "/commondir";
          isWorktree = builtins.pathExists commondir;
          mainGitDir = gitDir + "/${pkgs.lib.fileContents commondir}";
          worktree = pkgs.lib.optionalString isWorktree (
            pkgs.lib.dropString (builtins.stringLength (toString mainGitDir))
              (toString gitDir));
        in pkgs.runCommandNoCC "install_sh_timestamp" {
          git_dir = builtins.path {
            name = "sdk-git-dir";
            path = if isWorktree
                   then mainGitDir
                   else gitDir;
          };
          nativeBuildInputs = [ pkgs.git ];
          preferLocalBuild = true;
          allowSubstitutes = false;
        } ''
          cd $git_dir${worktree}
          git log -n 1 --pretty=format:%h-%cI -- public/install.sh > $out
        ''
      );
    in pkgs.lib.linuxOnly (pkgs.runCommandNoCC "install-sh-release" {
      inherit version;
      inherit (pkgs) isMaster;
      inherit revision;
      manifest = ./manifest.json;
      buildInputs = [ pkgs.jo install-sh-lint install-sh ];
    } ''
      set -Eeuo pipefail

      # Building the artifacts
      mkdir -p $out

      version_manifest_file=$out/manifest.json

      cp $manifest $version_manifest_file
      # we stamp the file with the revision
      substitute "${install-sh}/install.sh" $out/install.sh \
        --subst-var revision

      # Creating the manifest
      # We name is "_manifest.json" as opposed to "manifest.json" because we
      # also export a "manifest.json" (which has nothing to do with the
      # release)
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
}
