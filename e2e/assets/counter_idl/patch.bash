cat <<<"$(jq '.canisters.hello.main="counter_idl.mo"' dfx.json)" >dfx.json
