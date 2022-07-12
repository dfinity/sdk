cat <<<"$(jq '.canisters.hello_backend.main="v1.mo"' dfx.json)" >dfx.json
