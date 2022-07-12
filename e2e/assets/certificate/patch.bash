cat <<<"$(jq '.canisters.certificate.main="certificate.mo"' dfx.json)" >dfx.json
