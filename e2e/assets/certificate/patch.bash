cat <<<"$(jq '.canisters.certificate_backend.main="certificate.mo"' dfx.json)" >dfx.json
