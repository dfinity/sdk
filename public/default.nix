{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
, src ? null
, releaseVersion ? "latest"
  # TODO: Remove isMaster once switched to new CD system (https://dfinity.atlassian.net/browse/INF-1149)
, isMaster ? false
}:

let
  public = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../. "public");
  version = releaseVersion;
  gitDir = pkgs.lib.gitDir ../.;
in

rec {
  install-sh =
    pkgs.runCommandNoCC "install-sh" {
      inherit public;
      preferLocalBuild = true;
      allowSubstitutes = false;
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
        inherit version public;
        inherit isMaster;
        buildInputs = [ install-sh pkgs.shfmt pkgs.shellcheck ];
        preferLocalBuild = true;
        allowSubstitutes = false;
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
    pkgs.lib.linuxOnly (
      pkgs.runCommandNoCC "install-sh-release" {
        inherit version;
        inherit isMaster;

        # `revision` will be printed by `install.sh` as follows:
        #
        #   log "Executing DFINITY SDK install script, commit: @revision@"
        revision =
          if src != null
          then src.rev
          else pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);

        manifest = ./manifest.json;
        buildInputs = [ pkgs.jo install-sh-lint install-sh ];
        preferLocalBuild = true;
        allowSubstitutes = false;
      } ''
        set -Eeuo pipefail

        mkdir -p $out

        version_manifest_file=$out/manifest.json

        cp $manifest $version_manifest_file
        # we stamp the file with the revision
        substitute "${install-sh}/install.sh" $out/install.sh \
          --subst-var revision

        # Creating the manifest
        # We name it "_manifest.json" as opposed to "manifest.json" because we
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
      ''
    );
}
