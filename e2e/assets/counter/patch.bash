cat <<<"$(jq '.canisters.hello.main="counter.mo"' dfx.json)" >dfx.json
