jq '.canisters.hello_backend.main="greet.mo"' dfx.json | sponge dfx.json
