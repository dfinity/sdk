#!/dev/null

cat <<<"$(jq '.defaults.build.args="--compacting-gcX"' dfx.json)" >dfx.json
