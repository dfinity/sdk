{ pkgs ? import ./nix {}
, agent-js ? import ./src/agent/javascript { inherit pkgs; }
, bootstrap-js ? import ./src/bootstrap { inherit pkgs agent-js; }
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

  # Install agent
  mkdir $out/js-user-library
  tar xvzf ${agent-js.out}/dfinity-*.tgz --strip-component 1 --directory $out/js-user-library
  cp -R ${agent-js.lib}/node_modules $out/js-user-library

  # Install bootstrap
  mkdir $out/bootstrap
  cp -R ${bootstrap-js.out}/* $out/bootstrap/

  cp -R ${distributed-canisters} $out/canisters
''
