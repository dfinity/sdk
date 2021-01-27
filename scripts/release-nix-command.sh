# keep in mind: you cannot define variables here and then use them,
# because they won't be defined when envsubst runs.

echo "Switching to branch: $USER/release-$NEW_DFX_VERSION"
echo git switch -c $USER/release-$NEW_DFX_VERSION

echo "Updating version in src/dfx/Cargo.toml"
# update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION
sed -i '0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/' src/dfx/Cargo.toml

echo "Building dfx with cargo."
cargo build

echo "Appending version to public/manifest.json"
# Append the new version to `public/manifest.json` by appending it to the `versions` list.
cat <<<$(jq --indent 4 '.versions += ["$NEW_DFX_VERSION"]' public/manifest.json) >public/manifest.json

echo "Creating a pull request."
echo git add --all
echo git commit --signoff --message "chore: Release $NEW_DFX_VERSION"
echo git push origin $USER/release-$NEW_DFX_VERSION
