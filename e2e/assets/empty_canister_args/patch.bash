jq '.defaults.build.args="--error-detail 5 --compacting-gcX" | .canisters.e2e_project_backend.args=""' dfx.json | sponge dfx.json
