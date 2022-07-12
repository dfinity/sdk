cat <<<"$(jq '.canisters.hello.main="greet.mo"' dfx.json)" >dfx.json
