start_webserver() {
    local port script_dir
    script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    port=$(python3 "$script_dir/get_ephemeral_port.py")
    export E2E_WEB_SERVER_PORT="$port"

    python3 -m http.server "$E2E_WEB_SERVER_PORT" "$@" &
    export E2E_WEB_SERVER_PID=$!

    while ! nc -z localhost "$E2E_WEB_SERVER_PORT"; do
      sleep 1
    done
}

stop_webserver() {
    if [ "$E2E_WEB_SERVER_PID" ]; then
        kill "$E2E_WEB_SERVER_PID"
    fi
}
