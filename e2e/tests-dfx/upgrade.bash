#!/usr/bin/env bats

load ../utils/_

RANDOM_EMPHEMERAL_PORT=$(shuf -i 49152-65535 -n 1)

setup() {
    standard_setup
}

@test "upgrade succeeds (nix)" {
    [ "$GITHUB_WORKFLOW" ] && skip "skipping on github workflow"

    log "upgrade succeeds - start"
    latest_version="0.4.7"
    latest_version_dir="downloads/dfx/$latest_version/x86_64-$(uname -s | tr '[:upper:]' '[:lower:]')/"
    dfx_archive_file_name="dfx-$latest_version.tar.gz"
    mkdir -p "$latest_version_dir"
    cp "$(which dfx)" .
    version=$(./dfx --version)
    log "tar"
    tar -czf "$latest_version_dir/$dfx_archive_file_name" dfx
    echo '{
      "tags": {
        "latest": "0.4.7"
      },
      "versions": [
        "0.4.3",
        "0.4.4",
        "0.4.7"
      ]
    }' > manifest.json
    log "start python http server"
    python3 -m http.server "$RANDOM_EMPHEMERAL_PORT" &
    WEB_SERVER_PID=$!

    log "wait for http server"
    while ! nc -z localhost "$RANDOM_EMPHEMERAL_PORT"; do
        sleep 1
    done
    log "http server ready"

    # Override current version to force upgrade
    log "dfx update (1)"
    assert_command ./dfx upgrade \
        --current-version 0.4.6 \
        --release-root "http://localhost:$RANDOM_EMPHEMERAL_PORT"
    assert_match "Current version: .*"
    assert_match "Fetching manifest .*"
    assert_match "New version available: .*"

    log "dfx update (2)"
    assert_command ./dfx upgrade \
        --release-root "http://localhost:$RANDOM_EMPHEMERAL_PORT"
    assert_match "Already up to date"

    log "kill web server"

    kill "$WEB_SERVER_PID"
    assert_command ./dfx --version
    assert_match "$version"
}

@test "upgrade succeeds (github)" {
    [ "$NIX_STORE" ] && skip "skipping on nix"
    # on github, we can reach sdk.dfinity.org

    # Override current version to force upgrade
    log "dfx update (1)"
    assert_command ./dfx upgrade \
        --current-version 0.4.6
    assert_match "Current version: .*"
    assert_match "Fetching manifest .*"
    assert_match "New version available: .*"

    log "dfx update (2)"
    assert_command ./dfx upgrade
    assert_match "Already up to date"
}
