## 200_downloader.sh

define_flag_BOOL "insecure" "Allows downloading from insecure URLs, either using HTTP or TLS 1.2 or less."

check_help_for() {
    local _cmd
    local _arg
    local _ok
    _cmd="$1"
    _ok="y"
    shift

    # If we're running on OS-X, older than 10.13, then we always
    # fail to find these options to force fallback
    if check_cmd sw_vers; then
        if [ "$(sw_vers -productVersion | cut -d. -f2)" -lt 13 ]; then
            # Older than 10.13
            echo "Warning: Detected OS X platform older than 10.13"
            _ok="n"
        fi
    fi

    for _arg in "$@"; do
        if ! "$_cmd" --help | grep -q -- "$_arg"; then
            _ok="n"
        fi
    done

    test "$_ok" = "y"
}

# This wraps curl or wget. Try curl first, if not installed, use wget instead.
# Arguments:
#   $1 - URL to download.
#   $2 - Path to output the download. Use - to output to stdout.
downloader() {
    local _dld
    if check_cmd curl; then
        _dld=curl
    elif check_cmd wget; then
        _dld=wget
    else
        _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]; then
        need_cmd "$_dld"
    elif [ "$_dld" = curl ]; then
        if check_help_for curl --proto --tlsv1.3; then
            curl --proto '=https' --tlsv1.3 --silent --show-error --fail --location "$1" --output "$2"
        elif ! [ "$_flag_INSECURE" ]; then
            warn "Not forcing TLS v1.3, this is potentially less secure"
            curl --silent --show-error --fail --location "$1" --output "$2"
        else
            err "TLS 1.3 is not supported on this platform. To force using it, use the --insecure flag."
        fi
    elif [ "$_dld" = wget ]; then
        if check_help_for wget --https-only --secure-protocol; then
            wget --https-only --secure-protocol=TLSv1_3 "$1" -O "$2"
        elif ! [ "$_flag_INSECURE" ]; then
            warn "Not forcing TLS v1.3, this is potentially less secure"
            wget "$1" -O "$2"
        else
            err "TLS 1.3 is not supported on this platform. To force using it, use the --insecure flag."
        fi
    else
        err "Unknown downloader" # should not reach here
    fi
}
