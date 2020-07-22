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
    cargoTestCommands = _: [];
    override = oldAttrs: {
      DFX_ASSETS = assets-minimal;

      OPENSSL_STATIC = true;
      OPENSSL_LIB_DIR = "${pkgs.pkgsStatic.openssl.out}/lib";
      OPENSSL_INCLUDE_DIR = "${pkgs.pkgsStatic.openssl.dev}/include";
    };
  };

in
pkgs.lib.standaloneRust
  {
    drv = workspace.build;
    exename = "dfx";
    usePackager = false;
  }
