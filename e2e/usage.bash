load utils/_

@test "dfx help succeeds" {
  dfx --help
}

@test "dfx help contains new command" {
  dfx --help | grep new
}

@test "using an invalid command fails" {
    run dfx blurp
    if [[ $status == 0 ]]; then
        echo $@ >&2
        exit 1
    fi
}

@test "returns the right error if not in a project" {
    # Make sure we're in an empty directory.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    assert_command_fail dfx build
    assert_match "must be run in a project"

    dfx new t
    cd t
    assert_command dfx build
}
