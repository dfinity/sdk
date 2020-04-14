{ pkgs ? import ../nix { inherit system; }
, system ? builtins.currentSystem
, src ? builtins.fetchGit ../.
}:
let
  install =
    pkgs.runCommandNoCC "install.sh.template" {
      src = pkgs.lib.gitOnlySource ../. ./install;
      preferLocalBuild = true;
      allowSubstitutes = false;
      nativeBuildInputs = [ pkgs.shfmt pkgs.shellcheck ];
      allowedReferences = [];
    } ''
      echo "Running shellcheck..."
      shellcheck --shell=sh $src/*.sh \
        --exclude SC2039 \
        --exclude SC2154 \
        --exclude SC2034

      echo "Running shfmt..."
      shfmt -d -p -i 4 -ci -bn -s $src/*.sh

      echo "Creating output..."
      cat $src/*.sh > $out

      echo "Fixing up..."
      # Get rid of comments that don't start with '##' or '#!'.
      sed -i "
        /#!.*/p
        /##.*/p
        /^ *$/d
        /^ *#/d
        s/ *#.*//
      " $out
    '';
in
pkgs.lib.linuxOnly (
  pkgs.runCommandNoCC "install.sh" {
    # `revision` will be printed by `install.sh` as follows:
    #
    #   log "Executing DFINITY SDK install script, commit: @revision@"
    revision = src.rev;
    preferLocalBuild = true;
    allowSubstitutes = false;
    inherit install;
  } ''
    # we stamp the file with the revision
    substitute $install $out --subst-var revision
  ''
)
