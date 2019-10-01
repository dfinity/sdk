{   bats
,   coreutils
,   dfinity-sdk
,   runCommandNoCC
,   stdenv
,   killall
}:

runCommandNoCC "e2e-tests" {
    buildInputs = [ bats coreutils dfinity-sdk.packages.rust-workspace-debug stdenv.cc killall ];
} ''
    # We want $HOME/.cache to be in a new temporary directory.
    HOME=$(mktemp -d -t dfx-e2e-home-XXXX)

    timeout --preserve-status 120 bats --recursive ${../e2e}/* | tee $out
''
