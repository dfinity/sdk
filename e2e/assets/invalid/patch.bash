#!/dev/null

cat <<<"$(jq '.canisters.e2e_project_backend.main="invalid.mo"' dfx.json)" >dfx.json
