{   bats
,   coreutils
,   curl
,   dfinity-sdk
,   runCommandNoCC
,   stdenv
,   killall
}:

runCommandNoCC "e2e-tests" {
    buildInputs = [ bats coreutils curl dfinity-sdk.packages.rust-workspace-debug stdenv.cc killall ];
} ''
    # We want $HOME/.cache to be in a new temporary directory.
    HOME=$(mktemp -d -t dfx-e2e-home-XXXX)

    # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
    # less resources than a dev's computer, so e2e might take longer.
    timeout --preserve-status 600 bats --recursive ${../e2e}/* | tee $out
''
