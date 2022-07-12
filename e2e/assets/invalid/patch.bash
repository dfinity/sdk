#!/dev/null

cat <<<"$(jq '.canisters.e2e_project.main="invalid.mo"' dfx.json)" >dfx.json
