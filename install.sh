#!/usr/bin/env sh

##
## Borrowed from rustup (https://sh.rustup.rs)
##
## This is just a little script that can be downloaded from the internet to
## install dfx. It just does platform detection, downloads the installer
## and runs it.
##
## You are NOT AUTHORIZED to remove any license agreements or prompts from the following script.
##
set -u

# A newline separated list of boolean flags. See the read_flags function to see how it's parsed.
DFX_BOOL_FLAGS=""

# Make a BOOLEAN flag and its description.
#
# Arguments:
#   $1 - The long name of the boolean. This will be used on the command line. The name of the
#        environment variable will be `flag_NAME` where NAME is this argument, capitalized.
#        The value of this argument is empty string if not specified, and "1" if it is.
#   $2 - A description of the flag. This is not currently used but will be when we have enough
#        flags to implement help.
define_flag_BOOL() {
    local VARNAME
    VARNAME="flag_$(echo "$1" | tr /a-z/ /A-Z)"
    eval "$VARNAME=\${$VARNAME:-}"
    DFX_BOOL_FLAGS="${DFX_BOOL_FLAGS}--${1} $VARNAME $2"
}

# Get the flag name of a line in the flag description.
get_flag_name() {
    echo "$1"
}

# Get the variable name of a line in the flag description.
get_var_name() {
    echo "$2"
}

# Read all the command line flags and set the flag_XXXX environment variables.
#
# Arguments:
#   $* - Flags to parse.
# Side Effects:
#   Environment variables are set according to flags defined and flags.
read_flags() {
    # Set values from command line.
    # shellcheck disable=SC2199
    # https://github.com/koalaman/shellcheck/wiki/SC2199
    while [ -n "$*" ]; do
        local ARG="$1"
        shift

        OLD_IFS="$IFS"
        IFS=$'\n'
        for line in ${DFX_BOOL_FLAGS}; do
            [ "$line" ] || break

            IFS="$OLD_IFS"
            FLAG="$(get_flag_name "$line")"
            VARNAME="$(get_var_name "$line")"

            if [ "$ARG" == "$FLAG" ]; then
                eval "$VARNAME=1"
            fi
        done
    done
}

log() {
    if "$_ansi_escapes_are_valid"; then
        printf "\33[1minfo:\33[0m %s\n" "$1" 1>&2
    else
        printf '%s\n' "$1" 1>&2
    fi
}

say() {
    printf 'dfinity-sdk: %s\n' "$1"
}

warn() {
    if $_ansi_escapes_are_valid; then
        printf "\33[1mwarn:\33[0m %s\n" "$1" 1>&2
    else
        printf '%s\n' "$1" 1>&2
    fi
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

define_flag_BOOL "insecure" "Allows downloading from insecure URLs, either using HTTP or TLS 1.2 or less."

check_help_for() {
    local _arch
    local _cmd
    local _arg
    _arch="$1"
    shift
    _cmd="$1"
    shift

    local _category
    if "$_cmd" --help | grep -q 'For all options use the manual or "--help all".'; then
        _category="all"
    else
        _category=""
    fi

    case "$_arch" in

        *darwin*)
            if check_cmd sw_vers; then
                case $(sw_vers -productVersion) in
                    10.*)
                        # If we're running on macOS, older than 10.13, then we always
                        # fail to find these options to force fallback
                        if [ "$(sw_vers -productVersion | cut -d. -f2)" -lt 13 ]; then
                            # Older than 10.13
                            echo "Warning: Detected macOS platform older than 10.13"
                            return 1
                        fi
                        ;;

                        # We assume these will be OK for now
                    11.*) ;; # Big Sur
                    12.*) ;; # Monterey
                    13.*) ;; # Ventura
                    14.*) ;; # Sonoma

                    *)
                        # Unknown product version, warn and continue
                        echo "Warning: Detected unknown macOS major version: $(sw_vers -productVersion)"
                        echo "Warning TLS capabilities detection may fail"
                        ;;
                esac
            fi
            ;;

    esac

    for _arg in "$@"; do
        if ! "$_cmd" --help "$_category" | grep -q -- "$_arg"; then
            return 1
        fi
    done

    true # not strictly needed
}

is_zsh() {
    [ -n "${ZSH_VERSION-}" ]
}

# This wraps curl or wget. Try curl first, if not installed,
# use wget instead.
# Arguments:
#   $1 - URL to download.
#   $2 - Path to output the download. Use - to output to stdout.
#   $3 - The architecture, used to determine TLS capabilities.
downloader() {
    # zsh does not split words by default, Required for curl retry arguments below.
    is_zsh && setopt local_options shwordsplit

    local _dld
    local _ciphersuites
    local _err
    local _status
    local _retry
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
        check_curl_for_retry_support
        _retry="$RETVAL"
        get_ciphersuites_for_curl
        _ciphersuites="$RETVAL"
        if [ -n "$_ciphersuites" ]; then
            # shellcheck disable=SC2086 # $retry is intentionally split
            _err=$(curl $_retry --proto '=https' --tlsv1.2 --ciphers "$_ciphersuites" --silent --show-error --fail --location "$1" --output "$2" 2>&1)
            _status=$?
        else
            echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
            if ! check_help_for "$3" curl --proto --tlsv1.2; then
                echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                # shellcheck disable=SC2086 # $retry is intentionally split
                _err=$(curl $_retry --silent --show-error --fail --location "$1" --output "$2" 2>&1)
                _status=$?
            else
                # shellcheck disable=SC2086 # $retry is intentionally split
                _err=$(curl $_retry --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" --output "$2" 2>&1)
                _status=$?
            fi
        fi
        if [ -n "$_err" ]; then
            echo "$_err" >&2
            if echo "$_err" | grep -q 404$; then
                err "installer for platform '$3' not found, this may be unsupported"
            fi
        fi
        return $_status
    elif [ "$_dld" = wget ]; then
        if [ "$(wget -V 2>&1 | head -2 | tail -1 | cut -f1 -d" ")" = "BusyBox" ]; then
            echo "Warning: using the BusyBox version of wget.  Not enforcing strong cipher suites for TLS or TLS v1.2, this is potentially less secure"
            _err=$(wget "$1" -O "$2" 2>&1)
            _status=$?
        else
            get_ciphersuites_for_wget
            _ciphersuites="$RETVAL"
            if [ -n "$_ciphersuites" ]; then
                _err=$(wget --https-only --secure-protocol=TLSv1_2 --ciphers "$_ciphersuites" "$1" -O "$2" 2>&1)
                _status=$?
            else
                echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
                if ! check_help_for "$3" wget --https-only --secure-protocol; then
                    echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                    _err=$(wget "$1" -O "$2" 2>&1)
                    _status=$?
                else
                    _err=$(wget --https-only --secure-protocol=TLSv1_2 "$1" -O "$2" 2>&1)
                    _status=$?
                fi
            fi
        fi
        if [ -n "$_err" ]; then
            echo "$_err" >&2
            if echo "$_err" | grep -q ' 404 Not Found$'; then
                err "installer for platform '$3' not found, this may be unsupported"
            fi
        fi
        return $_status
    else
        err "Unknown downloader" # should not reach here
    fi
}

# Check if curl supports the --retry flag, then pass it to the curl invocation.
check_curl_for_retry_support() {
    local _retry_supported=""
    # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
    if check_help_for "notspecified" "curl" "--retry"; then
        _retry_supported="--retry 3"
        if check_help_for "notspecified" "curl" "--continue-at"; then
            # "-C -" tells curl to automatically find where to resume the download when retrying.
            _retry_supported="--retry 3 -C -"
        fi
    fi

    RETVAL="$_retry_supported"
}

# Return cipher suite string specified by user, otherwise return strong TLS 1.2-1.3 cipher suites
# if support by local tools is detected. Detection currently supports these curl backends:
# GnuTLS and OpenSSL (possibly also LibreSSL and BoringSSL). Return value can be empty.
get_ciphersuites_for_curl() {
    if [ -n "${RUSTUP_TLS_CIPHERSUITES-}" ]; then
        # user specified custom cipher suites, assume they know what they're doing
        RETVAL="$RUSTUP_TLS_CIPHERSUITES"
        return
    fi

    local _openssl_syntax="no"
    local _gnutls_syntax="no"
    local _backend_supported="yes"
    if curl -V | grep -q ' OpenSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' LibreSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' BoringSSL/'; then
        _openssl_syntax="yes"
    elif curl -V | grep -iq ' GnuTLS/'; then
        _gnutls_syntax="yes"
    else
        _backend_supported="no"
    fi

    local _args_supported="no"
    if [ "$_backend_supported" = "yes" ]; then
        # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
        if check_help_for "notspecified" "curl" "--tlsv1.2" "--ciphers" "--proto"; then
            _args_supported="yes"
        fi
    fi

    local _cs=""
    if [ "$_args_supported" = "yes" ]; then
        if [ "$_openssl_syntax" = "yes" ]; then
            _cs=$(get_strong_ciphersuites_for "openssl")
        elif [ "$_gnutls_syntax" = "yes" ]; then
            _cs=$(get_strong_ciphersuites_for "gnutls")
        fi
    fi

    RETVAL="$_cs"
}

# Return cipher suite string specified by user, otherwise return strong TLS 1.2-1.3 cipher suites
# if support by local tools is detected. Detection currently supports these wget backends:
# GnuTLS and OpenSSL (possibly also LibreSSL and BoringSSL). Return value can be empty.
get_ciphersuites_for_wget() {
    if [ -n "${RUSTUP_TLS_CIPHERSUITES-}" ]; then
        # user specified custom cipher suites, assume they know what they're doing
        RETVAL="$RUSTUP_TLS_CIPHERSUITES"
        return
    fi

    local _cs=""
    if wget -V | grep -q '\-DHAVE_LIBSSL'; then
        # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
        if check_help_for "notspecified" "wget" "TLSv1_2" "--ciphers" "--https-only" "--secure-protocol"; then
            _cs=$(get_strong_ciphersuites_for "openssl")
        fi
    elif wget -V | grep -q '\-DHAVE_LIBGNUTLS'; then
        # "unspecified" is for arch, allows for possibility old OS using macports, homebrew, etc.
        if check_help_for "notspecified" "wget" "TLSv1_2" "--ciphers" "--https-only" "--secure-protocol"; then
            _cs=$(get_strong_ciphersuites_for "gnutls")
        fi
    fi

    RETVAL="$_cs"
}

# Return strong TLS 1.2-1.3 cipher suites in OpenSSL or GnuTLS syntax. TLS 1.2
# excludes non-ECDHE and non-AEAD cipher suites. DHE is excluded due to bad
# DH params often found on servers (see RFC 7919). Sequence matches or is
# similar to Firefox 68 ESR with weak cipher suites disabled via about:config.
# $1 must be openssl or gnutls.
get_strong_ciphersuites_for() {
    if [ "$1" = "openssl" ]; then
        # OpenSSL is forgiving of unknown values, no problems with TLS 1.3 values on versions that don't support it yet.
        echo "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384"
    elif [ "$1" = "gnutls" ]; then
        # GnuTLS isn't forgiving of unknown values, so this may require a GnuTLS version that supports TLS 1.3 even if wget doesn't.
        # Begin with SECURE128 (and higher) then remove/add to build cipher suites. Produces same 9 cipher suites as OpenSSL but in slightly different order.
        echo "SECURE128:-VERS-SSL3.0:-VERS-TLS1.0:-VERS-TLS1.1:-VERS-DTLS-ALL:-CIPHER-ALL:-MAC-ALL:-KX-ALL:+AEAD:+ECDHE-ECDSA:+ECDHE-RSA:+AES-128-GCM:+CHACHA20-POLY1305:+AES-256-GCM"
    fi
}

DFXVM_GITHUB_LATEST_RELEASE_ROOT="${DFXVM_GITHUB_LATEST_RELEASE_ROOT:-https://github.com/dfinity/dfxvm/releases/latest/download}"
DFX_VERSION="${DFX_VERSION-}"
DFXVM_INIT_YES="${DFXVM_INIT_YES-}"

# The SHA and the time of the last commit that touched this file.
SCRIPT_COMMIT_DESC="89243f2268b6d0ec77d589b7b1b27af931f70edd"

download_and_install() {
    SHASUM="$1"

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    local _archive="dfxvm-${_arch}"
    local _tarball_filename="${_archive}.tar.gz"
    local _tarball_url="${DFXVM_GITHUB_LATEST_RELEASE_ROOT}/${_tarball_filename}"
    local _sha256_filename="${_tarball_filename}.sha256"
    local _sha256_url="${_tarball_url}.sha256"

    log "Downloading latest release..."
    ensure downloader "$_tarball_url" "${_tarball_filename}" "$_arch"
    ensure downloader "$_sha256_url" "${_sha256_filename}" "$_arch"

    log "Checking integrity of tarball..."
    ensure "$SHASUM" -c "${_sha256_filename}"

    ensure tar -xzf "${_tarball_filename}" --strip-components=1 "${_archive}/dfxvm"
    ensure chmod u+x dfxvm
    ensure mv dfxvm dfxvm-init

    command="./dfxvm-init"

    if [ -n "${DFX_VERSION}" ]; then
        command="${command} --dfx-version \"${DFX_VERSION}\""
    fi

    if [ -n "${DFXVM_INIT_YES}" ]; then
        command="${command} --yes"
    fi

    eval "${command}"
}

main() {
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

    # Read flags.
    read_flags "$@"

    log "Executing dfxvm install script, commit: $SCRIPT_COMMIT_DESC"

    downloader --check
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    if check_cmd sha256sum; then
        SHASUM=sha256sum
    elif check_cmd shasum; then
        SHASUM=shasum
    else
        err "need 'shasum' or 'sha256sum' (neither command found)"
    fi
    need_cmd rm
    need_cmd tar
    need_cmd gzip
    need_cmd touch

    local _dir

    if ! _dir="$(mktemp -d 2>/dev/null)"; then
        if ! _dir="$(mktemp -d -t dfinity-dfxvm)"; then
            err "failed to create temporary directory"
        fi
    fi

    ensure mkdir -p "${_dir}"

    (
        ensure cd "${_dir}" >/dev/null
        download_and_install "$SHASUM"
    )
    local _subshell_exit_code=$?

    ignore rm "${_dir}"/dfxvm*
    ignore rmdir "${_dir}"

    exit $_subshell_exit_code
}

## output is one of the following, which correspond to part of the release asset filenames:
##    aarch64-apple-darwin
##    x86_64-apple-darwin
##    aarch64-unknown-linux-gnu
##    x86_64-unknown-linux-gnu

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

    case "$_cputype" in

        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;

        arm64 | aarch64)
            _cputype=aarch64
            ;;

        *)
            err "unknown CPU type: $_cputype"
            ;;

    esac

    case "$_ostype" in

        Linux)
            _ostype=unknown-linux-gnu
            ;;

        Darwin)
            _ostype=apple-darwin
            ;;

        *)
            err "unrecognized OS type: $_ostype"
            ;;

    esac

    _arch="${_cputype}-${_ostype}"

    RETVAL="$_arch"
}

main "$@" || exit $?
