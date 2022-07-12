cat <<<"$(jq '.canisters.e2e_project.main="identity.mo"' dfx.json)" >dfx.json
