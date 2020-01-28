{ supportedSystems ? [ "x86_64-linux" "x86_64-darwin" ]
, scrubJobs ? true
, RustSec-advisory-db ? null
, isMaster ? true
}:
let
  pkgs = import ../nix {};
in
pkgs.ci ../.
  {
    inherit supportedSystems scrubJobs isMaster;
    rev = pkgs.lib.commitIdFromGitRepo (pkgs.lib.gitDir ../.);
    packageSetArgs = {
      inherit RustSec-advisory-db;
    };
  }
