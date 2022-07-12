cat <<<"$(jq '.canisters.hello_backend.main="greet.mo"' dfx.json)" >dfx.json
