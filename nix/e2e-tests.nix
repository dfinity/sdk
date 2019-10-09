{   bats
,   coreutils
,   curl
,   dfinity-sdk
,   runCommandNoCC
,   stdenv
,   killall
}:
let batslib = builtins.fetchGit {
    url = "ssh://git@github.com/ztombol/bats-support";
    # ref = "0.3.0";  # TODO
    rev = "24a72e14349690bcbf7c151b9d2d1cdd32d36eb1";
}; in

runCommandNoCC "e2e-tests" {
    buildInputs = [ bats batslib coreutils curl dfinity-sdk.packages.rust-workspace-debug stdenv.cc killall ];
} ''
    # We want $HOME/.cache to be in a new temporary directory.
    export HOME=$(mktemp -d -t dfx-e2e-home-XXXX)
    # We use BATSLIB in our scripts to find the root of the BATSLIB repo.
    export BATSLIB="${batslib}"

    # Timeout of 10 minutes is enough for now. Reminder; CI might be running with
    # less resources than a dev's computer, so e2e might take longer.
    timeout --preserve-status 600 bats --recursive ${../e2e}/* | tee $out
''
