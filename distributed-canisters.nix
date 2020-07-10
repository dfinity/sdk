{ pkgs ? import ./nix {}
, assets-minimal ? import ./assets-minimal.nix { inherit pkgs; }
, dfx-minimal ? import ./dfx-minimal.nix { inherit pkgs assets-minimal; }
}:
let
  distributed = lib.noNixFiles (lib.gitOnlySource ./. ./src/distributed);
  lib = pkgs.lib;

  workspace = pkgs.runCommandNoCC "distributed-canisters-workspace" {} ''
    # We want $HOME/.cache to be in a writable temporary directory.
    export HOME=$(mktemp -d -t dfx-distributed-canisters-home-XXXX)

    mkdir -p $out

    for source_root in ${distributed}/*; do
      canister_name=$(basename $source_root)

      build_dir=$out/$canister_name
      mkdir -p $build_dir
      cp -R $source_root/* $build_dir

      ( cd $build_dir ; DFX_ASSETS=${assets-minimal} ${dfx-minimal}/bin/dfx build --skip-manifest )

      if [ -f override.wasm ]
      then cp override.wasm canisters/$canister_name/$canister_name.wasm
      fi
    done
  '';

in
pkgs.runCommandNoCCLocal "distributed-canisters" {} ''
  for canister_root in ${workspace}/*; do
    canister_name=$(basename $canister_root)

    output_dir=$out/$canister_name
    mkdir -p $output_dir

    for ext in did wasm
    do
      cp $canister_root/canisters/$canister_name/$canister_name.$ext $output_dir
    done
  done
''
