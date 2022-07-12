cat <<<"$(jq '.canisters.e2e_project.main="main.mo"' dfx.json)" >dfx.json
