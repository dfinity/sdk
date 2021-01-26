    #git switch -c $USER/release-$NEW_DFX_VERSION

    # update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION
    sed -i '0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/' src/dfx/Cargo.toml

    #cargo build

    pwd
    # Append the new version to `public/manifest.json` by appending it to the `versions` list.
    cat <<<$(jq '.versions += ["$NEW_DFX_VERSION"]' public/manifest.json) >public/manifest.json
