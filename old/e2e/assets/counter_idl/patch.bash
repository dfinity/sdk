jq '.canisters.hello_backend.main="counter_idl.mo"' dfx.json | sponge dfx.json
