{ pkgs ? import ./nix {}
# , cargo anonymous function at /Users/ericswanson/d/sdk/icx-proxy.nix:1:1 called without required argument 'cargo', at /Users/ericswanson/d/sdk/assets.nix:3:15
, assets-minimal ? import ./assets-minimal.nix { inherit pkgs; }
}:
let
  agent-rs-src = builtins.fetchGit {
    #url = "file:///Users/ericswanson/d/agent-rs";
    url = "ssh://git@github.com/dfinity/agent-rs";
    ref = "ericswanson/1622-icx-proxy";
    rev = "81ecb6c6a9aaf86777423ccd480c7e4bc6cde805";
  };
  workspace = pkgs.runCommandNoCCLocal "build-agent-rs" {} ''
    mkdir -p $out
    cp -R ${agent-rs-src}/icx-proxy/* $out

  #  cd $out
  #  cargo build
  '';

  workspace2 = pkgs.buildDfinityRustPackage {
    name = "icx-proxy-workspace";
    src = workspace; # ${pkgs.agent-rs}/icx-proxy; # agent-rs-src;
    regexes = [
      ".*/assets/.*$"
      ".*\.rs$"
      ".*\.lalrpop$"
      ".*Cargo\.toml$"
      ".*Cargo\.lock$"
      "^.cargo/config$"
    ];
    cargoTestCommands = _: [];
  };

in
pkgs.runCommandNoCCLocal "copy-icx-proxy-binary" {} ''
    cp ${workspace2.build}/target/debug/icx-proxy $out
''