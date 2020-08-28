#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    mkdir home-for-test
    export HOME=$(pwd)/home-for-test

    dfx_new
    dfx_start
    # 8
}

teardown() {
    dfx_stop
    rm -rf home-for-test
}

@test "H1" {
}

@test "H2" {
}

@test "H3" {
}

@test "H4" {
}

@test "H5" {
}

@test "H6" {
}

@test "H7" {
}

@test "H8" {
}

@test "H9" {
}

@test "H10" {
}

@test "H11" {
}

@test "H12" {
}

@test "H13" {
}

@test "H14" {
}

@test "H15" {
}

@test "H16" {
}

@test "H17" {
}

@test "H18" {
}

@test "H19" {
}

@test "H20" {
}

@test "H21" {
}

@test "H22" {
}

@test "H23" {
}

@test "H24" {
}

@test "H25" {
}

@test "H26" {
}

@test "H27" {
}

@test "H28" {
}

@test "H29" {
}

@test "H30" {
}

@test "H31" {
}

@test "H32" {
}

@test "H33" {
}

@test "H34" {
}

@test "H35" {
}

@test "H36" {
}

@test "H37" {
}

@test "H38" {
}

@test "H39" {
}

@test "H40" {
}

@test "H41" {
}

@test "H42" {
}

@test "H43" {
}

@test "H44" {
}

@test "H45" {
}

@test "H46" {
}

@test "H47" {
}

@test "H48" {
}

@test "H49" {
}

@test "H50" {
}

@test "H51" {
}

@test "H52" {
}

@test "H53" {
}

@test "H54" {
}

@test "H55" {
}

@test "H56" {
}

@test "H57" {
}

@test "H58" {
}

@test "H59" {
}

@test "H60" {
}

@test "H61" {
}

@test "H62" {
}

@test "H63" {
}

@test "H64" {
}

@test "H65" {
}

@test "H66" {
}

@test "H67" {
}

@test "H68" {
}

@test "H69" {
}

@test "H70" {
}

@test "H71" {
}

@test "H72" {
}

@test "H73" {
}

@test "H74" {
}

@test "H75" {
}

@test "H76" {
}

@test "H77" {
}

@test "H78" {
}

@test "H79" {
}

@test "H80" {
}

@test "H81" {
}

@test "H82" {
}

@test "H83" {
}

@test "H84" {
}

@test "H85" {
}

@test "H86" {
}

@test "H87" {
}

@test "H88" {
}

@test "H89" {
}

@test "H90" {
}

@test "H91" {
}

@test "H92" {
}

@test "H93" {
}

@test "H94" {
}

@test "H95" {
}

@test "H96" {
}

@test "H97" {
}

@test "H98" {
}

@test "H99" {
}

