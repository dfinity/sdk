{ pkgs ? import ./nix {}
, distributed-canisters ? import ./distributed-canisters.nix { inherit pkgs; }
}:
let
  icx-proxy-standalone = pkgs.lib.standaloneRust {
    drv = pkgs.icx-proxy;
    exename = "icx-proxy";
    usePackager = false;
  };
  replica-bin = pkgs.sources."replica-${pkgs.system}";
  starter-bin = pkgs.sources."ic-starter-${pkgs.system}";
  looseBinaryCache = pkgs.runCommandNoCCLocal "loose-binary-cache" {} ''
    mkdir -p $out

    gunzip <${replica-bin} >$out/replica
    gunzip <${starter-bin} >$out/ic-starter
    cp -R ${pkgs.sources.motoko-base}/src $out/base
    cp ${pkgs.motoko}/bin/mo-doc $out
    cp ${pkgs.motoko}/bin/mo-ide $out
    cp ${pkgs.motoko}/bin/moc $out
    cp ${pkgs.ic-ref}/bin/* $out
    cp ${icx-proxy-standalone}/bin/icx-proxy $out
  '';
in
pkgs.runCommandNoCCLocal "assets" {} ''
  mkdir -p $out

  tar -czf $out/binary_cache.tgz -C ${looseBinaryCache}/ .

  tar -czf $out/assetstorage_canister.tgz -C ${distributed-canisters}/assetstorage/ .
  tar -czf $out/wallet_canister.tgz -C ${distributed-canisters}/wallet/ .
  tar -czf $out/ui_canister.tgz -C ${distributed-canisters}/ui/ .

''
