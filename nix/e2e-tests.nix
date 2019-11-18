{   bats
,   coreutils
,   curl
,   dfinity-sdk
,   netcat
,   runCommandNoCC
,   stdenv
,   ps
,   sources
,   jq
}:
runCommandNoCC "e2e-tests" {
    buildInputs = [ bats coreutils curl dfinity-sdk.packages.rust-workspace-debug stdenv.cc ps netcat jq];
} ''
    # We want $HOME/.cache to be in a new temporary directory.
    export HOME=$(mktemp -d -t dfx-e2e-home-XXXX)
    # We use BATSLIB in our scripts to find the root of the BATSLIB repo.
    export BATSLIB="${sources.bats-support}"

    # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
    # less resources than a dev's computer, so e2e might take longer.
    timeout --preserve-status 600 bats --recursive ${../e2e}/* | tee $out
''
