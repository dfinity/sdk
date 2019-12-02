## 100_log.sh

log() {
    if $_ansi_escapes_are_valid; then
        printf "\33[1minfo:\33[0m %s\n" "$1" 1>&2
    else
        printf '%s\n' "$1" 1>&2
    fi
}

say() {
    printf 'dfinity-sdk: %s\n' "$1"
}

err() {
    say "$1" >&2
    exit 1
}
