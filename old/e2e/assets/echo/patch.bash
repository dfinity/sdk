jq '.canisters.hello_backend.main="echo.mo"' dfx.json | sponge dfx.json
