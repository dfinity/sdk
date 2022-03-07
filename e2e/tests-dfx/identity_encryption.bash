#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
}

##
## dfx identity get-principal
##

@test "identity get-principal: different identities have different principal ids" {
    assert_command dfx identity new jose
    assert_command dfx identity new juana

    PRINCPAL_ID_JOSE=$(dfx --identity jose identity get-principal)
    PRINCPAL_ID_JUANA=$(dfx --identity juana identity get-principal)

    if [ "$PRINCPAL_ID_JOSE" -eq "$PRINCPAL_ID_JUANA" ]; then
      echo "IDs should not match: Jose '${PRINCPAL_ID_JOSE}' == Juana '${PRINCPAL_ID_JUANA}'..." | fail
    fi
}