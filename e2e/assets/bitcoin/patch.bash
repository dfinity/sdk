# unix socket domain names can only be so long.  An attempt to use $(pwd) here resulted in this error:
# path must be shorter than libc::sockaddr_un.sun_path
cat <<<"$(jq '.incoming_source.Path="'"/tmp/e2e-ic-btc-adapter.$$.socket"'"' testnet.config.json)" >testnet.config.json
