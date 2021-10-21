{ pkgs ? import ./nix {}
}:
let
  distributed = lib.noNixFiles (lib.gitOnlySource ./src/distributed);
  lib = pkgs.lib;

in
pkgs.runCommandNoCCLocal "distributed-canisters" {
  moc = pkgs.motoko;
  base = pkgs.sources.motoko-base;
} ''
  mkdir -p $out

  for canister_dir in $(find ${distributed} -mindepth 1 -maxdepth 1 -type d); do
    canister_name=$(basename $canister_dir)

    build_dir=$out/$canister_name
    mkdir -p $build_dir

    $moc/bin/moc \
       $canister_dir/Main.mo \
       -o $build_dir/$canister_name.did \
       --idl \
       --package base $base
    $moc/bin/moc \
       $canister_dir/Main.mo \
       -o $build_dir/$canister_name.wasm \
       -c --release \
       --package base $base
  done

  for canister_wasm in ${distributed}/*.wasm; do
    canister_name=$(basename -s .wasm $canister_wasm)
    canister_did=$(dirname $canister_wasm)/$canister_name.did

    build_dir=$out/$canister_name
    mkdir -p $build_dir

    cp $canister_wasm $out/$canister_name/$canister_name.wasm
    cp $canister_did $out/$canister_name/$canister_name.did
  done
''
