cat <<<"$(jq '.canisters.e2e_project_backend.main="packtool.mo"' dfx.json)" >dfx.json
