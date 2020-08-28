{ pkgs ? import ./nix {}
, bootstrap-js ? import ./nix/agent-js/bootstrap-js.nix { inherit pkgs; }
, distributed-canisters ? import ./distributed-canisters.nix { inherit pkgs; }
}:
pkgs.runCommandNoCCLocal "assets" {} ''
  mkdir -p $out

  cp ${pkgs.dfinity.ic-replica}/bin/replica $out
  cp ${pkgs.dfinity.ic-starter}/bin/ic-starter $out
  cp -R ${pkgs.motoko.base-src} $out/base
  cp ${pkgs.motoko.didc}/bin/didc $out
  cp ${pkgs.motoko.mo-doc}/bin/mo-doc $out
  cp ${pkgs.motoko.mo-ide}/bin/mo-ide $out
  cp ${pkgs.motoko.moc}/bin/moc $out

  # Install bootstrap
  mkdir $out/bootstrap
  cp -R ${bootstrap-js.out}/* $out/bootstrap/

  cp -R ${distributed-canisters} $out/canisters
''
