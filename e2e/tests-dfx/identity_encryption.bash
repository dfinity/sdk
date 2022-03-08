#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
}

# create identity with password succeeds and can be used afterwards
# create identity without a password
# create identity with an empty password gets rejected
# import identity without a password
# import identity and add a password
# export identity works without a password
# export identity works with a password
# rename identity works on identity with a password