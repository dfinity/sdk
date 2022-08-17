jq '.canisters.hello_backend.main="counter.mo"' dfx.json | sponge dfx.json
