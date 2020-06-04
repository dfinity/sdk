# We need to build some canisters with dfx and include them in $DFX_ASSETS.
# However, dfx needs some of the contents of $DFX_ASSETS in order to build
# even the simplest canister.
# This derivation provides the minimal assets required to build canisters
# that use the base library and do not have a frontend.
{ pkgs ? import ./nix {}
}:
pkgs.runCommandNoCCLocal "assets-minimal" {
  inherit (pkgs.motoko) didc rts stdlib;
  moc = pkgs.motoko.moc-bin;
} ''
  mkdir -p $out

  cp $moc/bin/moc $out
  cp $didc/bin/didc $out
  cp $rts/rts/mo-rts.wasm $out

  mkdir $out/base
  cp -R $stdlib/. $out/base
''
