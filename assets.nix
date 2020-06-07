{ pkgs ? import ./nix {}
, agent-js ? import ./src/agent/javascript { inherit pkgs; }
, assets-minimal ? import ./assets-minimal.nix { inherit pkgs; }
}:
pkgs.runCommandNoCCLocal "assets" {} ''
  mkdir -p $out

  cp -R ${assets-minimal}/* $out

  cp ${pkgs.dfinity.ic-replica}/bin/replica $out
  cp ${pkgs.motoko.mo-ide}/bin/mo-ide $out

  mkdir $out/js-user-library
  tar xvzf ${agent-js.out}/dfinity-*.tgz --strip-component 1 --directory $out/js-user-library
  cp -R ${agent-js.lib}/node_modules $out/js-user-library
''
