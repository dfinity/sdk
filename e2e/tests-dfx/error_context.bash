#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
    install_asset error_context
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "corrupt dfx.json" {
    echo "corrupt" >dfx.json
    assert_command_fail dfx deploy

    # The bare minimum is to mention the file
    assert_match "dfx.json"

    # It's nice to mention the full path to the file
    assert_match "$(pwd)/dfx.json"

    # The underlying cause
    assert_match "expected value at line 1 column 1"
}

@test "packtool missing" {
    dfx_start

    assert_command dfx canister create packtool_missing

    # shellcheck disable=SC2094
    cat <<<"$(jq '.defaults.build.packtool="not-a-valid-packtool and some parameters"' dfx.json)" >dfx.json


    assert_command_fail dfx build packtool_missing

    # expect to see the name of the packtool and the parameters
    assert_match '"not-a-valid-packtool" "and" "some" "parameters"'
    # expect to see the underlying cause
    assert_match "No such file or directory"
}

@test "moc missing" {
    dfx_start

    assert_command dfx canister create m_o_c_missing

    rm -f "$(dfx cache show)/moc"
    assert_command_fail dfx build m_o_c_missing

    # expect to see the name of the binary
    assert_match "moc"

    # expect to see the full path of the binary
    assert_match "$(dfx cache show)/moc"

    # expect to see the underlying cause
    assert_match "No such file or directory"
}

@test "npm is not installed" {
    dfx_start

    assert_command dfx canister create npm_missing

    touch package.json
    (
        DFX=$(whereis -q dfx)
        SAVE_PATH="$PATH"
        # commands needed by assert_command_fail:
        # export PATH="$(dirname "$(whereis -b -q mktemp)"):$(dirname "$(whereis -b -q rm)")"
        export PATH="$(whereis -b -q mktemp rm echo | xargs dirname | sort | uniq | tr '\n' ':')"
        echo "PATH is $PATH"
        assert_command_fail "$DFX" deploy npm_missing
        export PATH="$SAVE_PATH"

        # expect to see the npm command line
        assert_match '"npm" "run" "build"'
        # expect to see the name of the canister
        assert_match "npm_missing"
        # expect to see the underlying cause
        assert_match "No such file or directory"
    )
}

@test "missing asset source directory" {
    dfx_start

    assert_command dfx canister create asset_bad_source_path

    assert_command_fail dfx deploy asset_bad_source_path

    # expect to see the bad path
    assert_match "src/does/not/exist"
    # expect to see the name of the canister
    assert_match "asset_bad_source_path"
    # expect to see the underlying cause
    assert_match "No such file or directory"
}

@test "custom bad build step" {
    dfx_start

    assert_command dfx canister create custom_bad_build_step

    assert_command_fail dfx build custom_bad_build_step

    # expect to see what it tried to call
    assert_match "not-the-name-of-an-executable-that-exists"
    # expect to see the name of the canister
    assert_match "custom_bad_build_step"
    # expect to see the underlying cause
    assert_match "No such file or directory"
}
