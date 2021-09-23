#!/usr/bin/env bats

load ../utils/_

RANDOM_EMPHEMERAL_PORT=$(shuf -i 49152-65535 -n 1)

setup() {
    standard_setup
}

@test "upgrade succeeds" {
    latest_version="0.4.7"
    latest_version_dir="downloads/dfx/$latest_version/x86_64-$(uname -s | tr '[:upper:]' '[:lower:]')/"
    dfx_archive_file_name="dfx-$latest_version.tar.gz"
    mkdir -p "$latest_version_dir"
    cp "$(which dfx)" .
    version=$(./dfx --version)
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
    python -m http.server "$RANDOM_EMPHEMERAL_PORT" &
    WEB_SERVER_PID=$!

    while ! nc -z localhost "$RANDOM_EMPHEMERAL_PORT"; do
        sleep 1
    done

    # Override current version to force upgrade
    assert_command ./dfx upgrade \
        --current-version 0.4.6 \
        --release-root "http://localhost:$RANDOM_EMPHEMERAL_PORT"
    assert_match "Current version: .*"
    assert_match "Fetching manifest .*"
    assert_match "New version available: .*"

    assert_command ./dfx upgrade \
        --release-root "http://localhost:$RANDOM_EMPHEMERAL_PORT"
    assert_match "Already up to date"

    kill "$WEB_SERVER_PID"
    assert_command ./dfx --version
    assert_match "$version"
}
