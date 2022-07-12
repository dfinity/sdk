cat <<<"$(jq '.canisters.e2e_project.main="packtool.mo"' dfx.json)" >dfx.json
