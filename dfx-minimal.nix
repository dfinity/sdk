{ pkgs ? import ./nix {}
, assets-minimal ? import ./assets-minimal.nix { inherit pkgs; }
}:
let
  workspace = pkgs.buildDfinityRustPackage {
    repoRoot = ./.;
    name = "dfx-minimal-workspace";
    srcDir = ./.;
    regexes = [
      ".*/assets/.*$"
      ".*\.rs$"
      ".*\.lalrpop$"
      ".*Cargo\.toml$"
      ".*Cargo\.lock$"
      "^.cargo/config$"
    ];
    cargoTestCommands = _: [
    ];
    override = oldAttrs: {
      DFX_ASSETS = assets-minimal;
    };
  };

in
pkgs.lib.standaloneRust
{
  drv = workspace.build;
  exename = "dfx";
  usePackager = false;
}
