#!/usr/bin/env sh

# Borrowed from rustup (https://sh.rustup.rs)

# This is just a little script that can be downloaded from the internet to
# install the DFINITY SDK. It just does platform detection, downloads the installer
# and runs it.

# You are NOT AUTHORIZED to remove any license agreements or prompts from the following script.

set -u

# If DFX_RELEASE_ROOT is unset or empty, default it.
SDK_WEBSITE="https://sdk.dfinity.org"
DFX_RELEASE_ROOT="${DFX_RELEASE_ROOT:-$SDK_WEBSITE/downloads/dfx/latest}"

sdk_install_dir() {
    if [ "${DFX_INSTALL_ROOT:-}" ]; then
        # By default we install to a home directory.
        printf %s "${DFX_INSTALL_ROOT}"
    elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
        printf %s /usr/local/bin
    elif [ "$(uname -s)" = Darwin ]; then
        # OS X does not allow users to write to /usr/bin by default. In case the
        # user is missing a /usr/local/bin we need to create it. Even if it is
        # not "writeable" we expect the user to have access to super user
        # privileges during the installation.
        mkdir -p /usr/local/bin 2>/dev/null || sudo mkdir -p /usr/local/bin || true
        printf %s /usr/local/bin
    elif [ -d /usr/bin ] && [ -w /usr/bin ]; then
        printf %s /usr/bin
    else
        # This is our last choice.
        printf %s "${HOME}/bin"
    fi
}

main() {
    downloader --check
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd tar

    if ! confirm_license; then
        echo "Please accept the license to continue."
        exit
    fi

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    # TODO: dfx can't yet be distributed as a single file, it needs supporting libraries
    # thus, make sure this handles archives
    local _dfx_url="${DFX_RELEASE_ROOT}/${_arch}/dfx-latest.tar.gz"

    local _dir
    _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t dfinity-sdk)"
    local _dfx_archive="${_dir}/dfx.tar.gz"
    local _dfx_file="${_dir}/dfx"

    _ansi_escapes_are_valid=false
    if [ -t 2 ]; then
        if [ "${TERM+set}" = 'set' ]; then
            case "$TERM" in
                xterm* | rxvt* | urxvt* | linux* | vt*)
                    _ansi_escapes_are_valid=true
                    ;;
            esac
        fi
    fi

    log "Creating uninstall script in ~/.cache/dfinity"
    mkdir -p "${HOME}/.cache/dfinity/"
    # Ensure there is a way to uninstall dfinity sdk.
    install_uninstall_script

    log "Checking for latest release..."

    ensure mkdir -p "$_dir"
    ensure downloader "$_dfx_url" "$_dfx_archive"
    tar -xf "$_dfx_archive" -O >"$_dfx_file"
    ensure chmod u+x "$_dfx_file"

    local _install_dir
    _install_dir="$(sdk_install_dir)"
    printf "%s\n" "Will install in: ${_install_dir}"
    mkdir -p "${_install_dir}" || true

    mv "$_dfx_file" "${_install_dir}" || sudo mv "$_dfx_file" "${_install_dir}" \
        || err "Failed to install the DFINITY Developement Kit: please check your permissions and try again."

    log "Installed $_install_dir/dfx"

    ignore rm -rf "$_dir"
}

get_architecture() {
    local _ostype _cputype _arch
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"

    if [ "$_ostype" = Darwin ] && [ "$_cputype" = i386 ]; then
        # Darwin `uname -m` lies
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            _cputype=x86_64
        fi
    fi

    case "$_ostype" in

        Linux)
            _ostype=linux
            ;;

        Darwin)
            _ostype=darwin
            ;;

        *)
            err "unrecognized OS type: $_ostype"
            ;;

    esac

    case "$_cputype" in

        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;

        *)
            err "unknown CPU type: $_cputype"
            ;;

    esac

    _arch="${_cputype}-${_ostype}"

    RETVAL="$_arch"
}

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

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    "$@"
}

# This wraps curl or wget. Try curl first, if not installed,
# use wget instead.
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
        if ! check_help_for curl --proto --tlsv1.2; then
            echo "Warning: Not forcing TLS v1.2, this is potentially less secure"
            curl --silent --show-error --fail --location "$1" --output "$2"
        else
            curl --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" --output "$2"
        fi
    elif [ "$_dld" = wget ]; then
        if ! check_help_for wget --https-only --secure-protocol; then
            echo "Warning: Not forcing TLS v1.2, this is potentially less secure"
            wget "$1" -O "$2"
        else
            wget --https-only --secure-protocol=TLSv1_2 "$1" -O "$2"
        fi
    else
        err "Unknown downloader" # should not reach here
    fi
}

install_uninstall_script() {
    set +u
    local uninstall_file_path
    local uninstall_script

    uninstall_script=$(
        cat <<'EOF'
#!/usr/bin/env sh

uninstall() {
    check_rm "${DFX_INSTALL_ROOT}/dfx"
    check_rm "${HOME}/bin/dfx"
    check_rm /usr/local/bin/dfx /usr/bin/dfx

    # Now clean the cache.
    clean_cache
}

check_rm() {
    local file
    for file in "$@"
    do
        [ -e "${file}" ] && rm "${file}"
    done
}

clean_cache() {
    # Check if home is unset or set to empty.
    if [ -z "$HOME" ]; then
        exit "HOME environment variable unset."
    fi

    rm -Rf "${HOME}/.cache/dfinity"
}

uninstall
EOF
    )
    set -u
    # Being a bit more paranoid and rechecking.
    assert_nz "${HOME}"
    uninstall_file_path="${HOME}/.cache/dfinity/uninstall.sh"
    log "uninstall path=${uninstall_file_path}"
    touch "${uninstall_file_path}"
    printf "%s" "$uninstall_script" >"${uninstall_file_path}"
    ensure chmod u+x "${uninstall_file_path}"
}

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

confirm_license() {
    local prompt header license
    header="\n DFINITY SDK \n Please READ the following license: \n\n"

    license="DFINITY Foundation -- All rights reserved. This is an ALPHA version
of the DFINITY Canister Software Development Kit (SDK). Permission is hereby granted
to use AS IS and only subject to the Alpha DFINITY Canister SDK License Agreement which
can be found here [https://sdk.dfinity.org/sdk-license-agreement]. It comes with NO WARRANTY.\n"

    prompt='Do you agree and wish to install the DFINITY ALPHA SDK [y/N]?'

    # we test if there is a terminal present (that is, STDIN is a TTY)
    if ! [ -t 0 ]; then
        printf "%s\n" "Please run in an interactive terminal."
        printf "%s\n" "Hint: Run  sh -ci \"\$(curl -fsSL $SDK_WEBSITE/install.sh)\""
        exit 0
    fi
    printf "%b" "$header"
    printf "%b\n\n" "$license"
    printf "%b\n" "$prompt"
    while true; do
        read -r resp
        case "$resp" in
            # Continue on yes or y.
            [Yy][Ee][Ss] | [Yy])
                return 0
                ;;
            # Exit on no or n
            [Nn][Oo] | [Nn])
                return 1
                ;;
            *)
                # invalid input
                # Send out an ANSI escape code to move up and then to delete the
                # line. Keeping it separate for convenience.
                printf "%b\n" "\033[2A"
                printf "%b " "\r\033[KAnswer with a yes or no to continue. [y/N]"
                ;;
        esac
    done
}

main "$@" || exit 1
