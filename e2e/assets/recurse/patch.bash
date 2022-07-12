cat <<<"$(jq '.canisters.hello.main="recurse.mo"' dfx.json)" >dfx.json
