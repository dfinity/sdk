cat <<<"$(jq '.canisters.hello_backend.main="counter_idl.mo"' dfx.json)" >dfx.json
