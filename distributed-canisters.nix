{ pkgs ? import ./nix {}
}:
let
  distributed = lib.noNixFiles (lib.gitOnlySource ./. ./src/distributed);
  lib = pkgs.lib;

in
pkgs.runCommandNoCCLocal "distributed-canisters" {
  inherit (pkgs.motoko) didc rts;
  moc = pkgs.motoko.moc-bin;
  base = pkgs.motoko.base-src;
} ''
  mkdir -p $out

  for canister_mo in ${distributed}/*.mo; do
    canister_name=$(basename -s .mo $canister_mo)

    build_dir=$out/$canister_name
    mkdir -p $build_dir

    $moc/bin/moc \
       $canister_mo \
       -o $build_dir/$canister_name.did \
       --idl \
       --package base $base
    MOC_RTS=$rts/rts/mo-rts.wasm $moc/bin/moc \
       $canister_mo \
       -o $build_dir/$canister_name.wasm \
       -c --debug \
       --package base $base
  done
''
