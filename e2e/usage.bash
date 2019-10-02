#!/usr/bin/env bats

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
