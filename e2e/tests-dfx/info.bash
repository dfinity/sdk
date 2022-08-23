#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "displays the default webserver port for the local shared network" {
    assert_command dfx info webserver-port
    assert_eq "4943"
}

@test "displays the webserver port for a project-specific network" {
    define_project_network
    assert_command dfx info webserver-port
    assert_eq "8000"
}
