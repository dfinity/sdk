# We need to build some canisters with dfx and include them in $DFX_ASSETS.
# However, dfx needs some of the contents of $DFX_ASSETS in order to build
# even the simplest canister.
# This derivation provides the minimal assets required to build canisters
# that use the base library and do not have a frontend.
{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
}:
let
  inputs = with pkgs; [
    bash
    coreutils
  ];

in
builtins.derivation {
  name = "assets-minimal";
  system = pkgs.stdenv.system;
  PATH = pkgs.lib.makeSearchPath "bin" inputs;
  builder =
    pkgs.writeScript "builder.sh" ''
      #!${pkgs.stdenv.shell}
      set -eo pipefail

      mkdir -p $out

      cp ${pkgs.motoko.moc-bin}/bin/moc $out
      cp ${pkgs.motoko.didc}/bin/didc $out
      cp ${pkgs.motoko.rts}/rts/mo-rts.wasm $out

      mkdir $out/base && cp -R ${pkgs.motoko.stdlib}/. $out/base
    '';
} // { meta = {}; }
