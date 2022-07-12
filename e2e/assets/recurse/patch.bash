cat <<<"$(jq '.canisters.hello_backend.main="recurse.mo"' dfx.json)" >dfx.json
