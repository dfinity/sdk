#!/dev/null

cat <<<"$(jq '.canisters.e2e_project_backend.args="--compacting-gcY"' dfx.json)" >dfx.json
cat <<<"$(jq '.defaults.build.args="--compacting-gcX"' dfx.json)" >dfx.json
