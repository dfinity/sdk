{ pkgs ? import ./nix {}
, distributed-canisters ? import ./distributed-canisters.nix { inherit pkgs; }
}:
let
  ic-btc-adapter-bin = pkgs.sources."ic-btc-adapter-${pkgs.system}";
  replica-bin = pkgs.sources."replica-${pkgs.system}";
  canister-sandbox-bin = pkgs.sources."canister-sandbox-${pkgs.system}";
  sandbox-launcher-bin = pkgs.sources."sandbox-launcher-${pkgs.system}";
  starter-bin = pkgs.sources."ic-starter-${pkgs.system}";
  looseBinaryCache = pkgs.runCommandNoCCLocal "loose-binary-cache" {} ''
    mkdir -p $out

    gunzip <${ic-btc-adapter-bin} >$out/ic-btc-adapter
    gunzip <${replica-bin} >$out/replica
    gunzip <${canister-sandbox-bin} >$out/canister_sandbox
    gunzip <${sandbox-launcher-bin} >$out/sandbox_launcher
    gunzip <${starter-bin} >$out/ic-starter
    cp -R ${pkgs.sources.motoko-base}/src $out/base
    cp ${pkgs.motoko}/bin/mo-doc $out
    cp ${pkgs.motoko}/bin/mo-ide $out
    cp ${pkgs.motoko}/bin/moc $out
    cp ${pkgs.ic-ref}/bin/* $out
    cp ${pkgs.icx-proxy}/bin/icx-proxy $out
  '';
in
pkgs.runCommandNoCCLocal "assets" {} ''
  mkdir -p $out

  tar -czf $out/binary_cache.tgz -C ${looseBinaryCache}/ .

  tar -czf $out/assetstorage_canister.tgz -C ${distributed-canisters}/assetstorage/ .
  tar -czf $out/wallet_canister.tgz -C ${distributed-canisters}/wallet/ .
  tar -czf $out/ui_canister.tgz -C ${distributed-canisters}/ui/ .

''
