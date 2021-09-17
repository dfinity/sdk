load ../utils/_

teardown() {
    dfx_stop
}

@test "dfx help succeeds" {
  dfx --help
}

@test "dfx help contains new command" {
  dfx --help | grep new
}

@test "using an invalid command fails" {
    run dfx blurp
    if [[ $status == 0 ]]; then
        echo "$@" >&2
        exit 1
    fi
}

@test "returns the right error if not in a project" {
    # Make sure we're in an empty directory.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    assert_command_fail dfx build
    assert_match "dfx.json not found, using default"

    dfx new t --no-frontend
    cd t
    dfx_start
    dfx canister create --all
    assert_command dfx build
}
