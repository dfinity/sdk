{ system ? builtins.currentSystem
, pkgs ? import ./nix { inherit system isMaster labels; }
, src ? builtins.fetchGit ./.
, releaseVersion ? "latest"
, RustSec-advisory-db ? pkgs.sources.advisory-db
, isMaster ? true
, labels ? {}
}:
rec {
  inherit (pkgs) nix-fmt nix-fmt-check;

  install = import ./public { inherit pkgs src; };

  # licenses = {
  #  dfx = pkgs.lib.runtime.runtimeLicensesReport dfx.build;
  # };
}
