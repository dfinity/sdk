jq '.canisters.hello_backend.main="call.mo"' dfx.json | sponge dfx.json
