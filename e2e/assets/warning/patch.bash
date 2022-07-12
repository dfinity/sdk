cat <<<"$(jq '.canisters.e2e_project_backend.main="main.mo"' dfx.json)" >dfx.json
