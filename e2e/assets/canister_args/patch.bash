jq '.canisters.e2e_project_backend.args="--compacting-gcY" | .defaults.build.args="--compacting-gcX"' dfx.json | sponge dfx.json
