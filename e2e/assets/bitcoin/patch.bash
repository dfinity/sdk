# shellcheck disable=SC2094
cat <<<"$(jq '.defaults.bitcoin.btc_adapter_config="'"$(pwd)/testnet.config.json"'"' dfx.json)" >dfx.json
# unix socket domain names can only be so long.  An attempt to use $(pwd) here resulted in this error:
# path must be shorter than libc::sockaddr_un.sun_path
cat <<<"$(jq '.incoming_source.Path="'"/tmp/e2e-ic-btc-adapter.$$.socket"'"' testnet.config.json)" >testnet.config.json
