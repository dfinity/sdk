cat <<<"$(jq '.canisters.hello.main="v1.mo"' dfx.json)" >dfx.json
