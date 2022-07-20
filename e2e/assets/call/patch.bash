cat <<<"$(jq '.canisters.hello_backend.main="call.mo"' dfx.json)" >dfx.json
