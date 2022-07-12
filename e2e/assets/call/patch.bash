cat <<<"$(jq '.canisters.hello.main="call.mo"' dfx.json)" >dfx.json
