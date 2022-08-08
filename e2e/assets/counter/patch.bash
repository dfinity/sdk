cat <<<"$(jq '.canisters.hello_backend.main="counter.mo"' dfx.json)" >dfx.json
