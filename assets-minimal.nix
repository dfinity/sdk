# We need to build some canisters with dfx and include them in $DFX_ASSETS.
# However, dfx needs some of the contents of $DFX_ASSETS in order to build
# even the simplest canister.
# This derivation provides the minimal assets required to build canisters
# that use the base library and do not have a frontend.
{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
}:
let
in
pkgs.runCommandNoCCLocal "assets-minimal" {
  inherit (pkgs.motoko) didc rts stdlib;
} ''
  mkdir -p $out

  cp ${pkgs.motoko.moc-bin}/bin/moc $out
  cp $didc/bin/didc $out
  cp $rts/rts/mo-rts.wasm $out

  mkdir $out/base
  cp -R $stdlib/. $out/base
''
