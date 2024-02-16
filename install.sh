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

#   Functions useful for dealing with the manifest (which is JSON).

# Get the version of a tag from the manifest JSON file.
# Arguments:
#   $1 - The tag to get.
#   STDIN - The manifest file.
# Returns:
#   0 if the tag was found, 1 if it wasn't.
#   Prints out the version number.
get_tag_from_manifest_json() {
    # Find the tag in the file. Then get the last digits.
    # The first grep returns `"tag_name": "1.2.3` (without the last quote).
    cat \
        | tr -d '\n' \
        | grep -o "\"$1\":[[:space:]]*\"[a-zA-Z0-9.]*" \
        | grep -o "[0-9.]*$"
}

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
    local _cmd
    local _arg
    local _ok
    _cmd="$1"
    _ok="y"
    shift

    # If we're running on OS-X, older than 10.13, then we always
    # fail to find these options to force fallback
    if check_cmd sw_vers; then
        case "$(sw_vers -productVersion)" in
            10.15*) ;; # Catalina
            11.*) ;;   # Big Sur
            12.*) ;;   # Monterey
            13.*) ;;   # Ventura
            *)
                warn "Detected OS X platform older than 10.15 (Catalina)"
                _ok="n"
                ;;
        esac
    fi

    for _arg in "$@"; do
        if ! "$_cmd" --help all | grep -q -- "$_arg"; then
            _ok="n"
        fi
    done

    test "$_ok" = "y"
}

# Check for an error message in the output of a command.
# Arguments:
#   $1 - The error message to look for.
#   $2... - The command and arguments to run.
# Returns:
#   Whether false if the error message was not found, or true if it wasn't (so the feature is
#   supported.
# TODO: move this logic to execute once during install.sh run.
check_support_for() {
    local err="$1"
    shift
    local cmd="$*"

    # Run the command, grep for the error message, if it is found returns false, if it
    # is not found, returns true.
    ! ($cmd 2>&1 | grep "$err" >/dev/null)
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
        if check_help_for curl --proto --tlsv1.2; then
            curl --proto '=https' --tlsv1.2 --show-error --fail --connect-timeout 10 --retry 5 --location "$1" --output "$2"
        elif ! [ "$flag_INSECURE" ]; then
            warn "Not forcing TLS v1.2, this is potentially less secure"
            curl --show-error --fail --connect-timeout 10 --retry 5 --location "$1" --output "$2"
        else
            err "TLS 1.2 is not supported on this platform. To force using it, use the --insecure flag."
        fi
    elif [ "$_dld" = wget ]; then
        if check_help_for wget --https-only --secure-protocol; then
            wget --https-only --secure-protocol=TLSv1_2 --timeout 10 --tries 5 --waitretry 5 "$1" -O "$2"
        elif ! [ "$flag_INSECURE" ]; then
            warn "Not forcing TLS v1.2, this is potentially less secure"
            wget --timeout 10 --tries 5 --waitretry 5 "$1" -O "$2"
        else
            err "TLS 1.2 is not supported on this platform. To force using it, use the --insecure flag."
        fi
    else
        err "Unknown downloader" # should not reach here
    fi
}

# If DFX_RELEASE_ROOT is unset or empty, default it.
SDK_WEBSITE="https://sdk.dfinity.org"
DFX_RELEASE_ROOT="${DFX_RELEASE_ROOT:-$SDK_WEBSITE/downloads/dfx}"
DFX_GITHUB_RELEASE_ROOT="${DFX_GITHUB_RELEASE_ROOT:-https://github.com/dfinity/sdk/releases/download}"
DFX_MANIFEST_JSON_URL="${DFX_MANIFEST_JSON_URL:-$SDK_WEBSITE/manifest.json}"
DFX_VERSION="${DFX_VERSION-}"

# The SHA and the time of the last commit that touched this file.
SCRIPT_COMMIT_DESC="df4b6282444975a9bd8c7a74ebb219ce0a1f8c9b"

# Get the version of a tag from the manifest JSON file.
# Arguments:
#   $1 - The tag to get.
#   STDIN - The manifest file.
# Returns:
#   0 if the tag was found, 1 if it wasn't.
#   Prints out the version number.
get_tag_from_manifest_json() {
    # Find the tag in the file. Then get the last digits.
    # The first grep returns `"tag_name": "1.2.3` (without the last quote).
    cat \
        | tr -d '\n' \
        | grep -o "\"$1\":[[:space:]]*\"[a-zA-Z0-9.]*" \
        | grep -o "[0-9.]*$"
}

get_manifest_version() {
    local _version
    _version="$(downloader "${DFX_MANIFEST_JSON_URL}" - | get_tag_from_manifest_json latest)" || return 2

    printf %s "${_version}"
}

validate_install_dir() {
    local dir="${1%/}"

    # We test it's a directory and writeable.

    # Try to create the directory if it doesn't exist already
    if ! [ -d "$dir" ]; then
        if ! mkdir -p "$dir"; then
            if type sudo >/dev/null; then
                sudo mkdir -p "$dir"
            fi
        fi
    fi

    ! [ -d "$dir" ] && return 1
    ! [ -w "$dir" ] && return 2

    # We also test it's in the $PATH of the user.
    case ":$PATH:" in
        *:$dir:*) ;;
        *) return 3 ;;
    esac

    return 0
}

sdk_install_dir() {
    if [ "${DFX_INSTALL_ROOT-}" ]; then
        # If user specifies an actual dir, use that.
        validate_install_dir "${DFX_INSTALL_ROOT}"
        printf %s "${DFX_INSTALL_ROOT}"
    elif validate_install_dir /usr/local/bin; then
        printf %s /usr/local/bin
    elif [ "$(uname -s)" = Darwin ]; then
        # OS X does not allow users to write to /usr/bin by default. In case the
        # user is missing a /usr/local/bin we need to create it. Even if it is
        # not "writeable" we expect the user to have access to super user
        # privileges during the installation.
        validate_install_dir /usr/local/bin
        printf %s /usr/local/bin
    elif validate_install_dir /usr/bin; then
        printf %s /usr/bin
    else
        # This is our last choice.
        printf %s "${HOME}/bin"
    fi
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

    log "Executing dfx install script, commit: $SCRIPT_COMMIT_DESC"

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

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    # Download the manifest if we need to.
    if [ -z "${DFX_VERSION}" ]; then
        DFX_VERSION=$(get_manifest_version)
    fi

    # TODO: dfx can't yet be distributed as a single file, it needs supporting libraries
    # thus, make sure this handles archives
    log "Version found: $DFX_VERSION"
    case "$DFX_VERSION" in
        0.[0-9].*)
            local _dfx_tarball_filename="dfx-${DFX_VERSION}.tar.gz"
            local _dfx_url="${DFX_RELEASE_ROOT}/${DFX_VERSION}/${_arch}/${_dfx_tarball_filename}"
            local _dfx_sha256_filename=""
            ;;

        *)
            local _dfx_tarball_filename="dfx-${DFX_VERSION}-${_arch}.tar.gz"
            local _dfx_url="${DFX_GITHUB_RELEASE_ROOT}/${DFX_VERSION}/${_dfx_tarball_filename}"
            local _dfx_sha256_filename="${_dfx_tarball_filename}.sha256"
            local _dfx_sha256_url="${_dfx_url}.sha256"
            ;;
    esac

    local _dir
    _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t dfinity-sdk)"
    local _dfx_archive="${_dir}/${_dfx_tarball_filename}"
    local _dfx_file="${_dir}/dfx"

    log "Creating uninstall script in ~/.cache/dfinity"
    mkdir -p "${HOME}/.cache/dfinity/"
    # Ensure there is a way to uninstall dfx
    install_uninstall_script

    log "Checking for latest release..."

    ensure mkdir -p "$_dir"
    ensure downloader "$_dfx_url" "$_dfx_archive"
    if [ -n "${_dfx_sha256_filename}" ]; then
        log "Checking integrity of tarball..."
        ensure downloader "$_dfx_sha256_url" "${_dir}/${_dfx_sha256_filename}"
        (
            ensure cd "${_dir}" >/dev/null
            ensure $SHASUM -c "${_dfx_sha256_filename}"
        )
    fi
    tar -xf "$_dfx_archive" -O >"$_dfx_file"
    ensure chmod u+x "$_dfx_file"

    local _install_dir
    _install_dir="$(sdk_install_dir)"
    printf "%s\n" "Will install in: ${_install_dir}"
    mkdir -p "${_install_dir}" || true

    if [ -w "${_install_dir}" ]; then
        MV="mv"
    else
        if ! type sudo >/dev/null; then
            err "Install directory '${_install_dir}' not writable and sudo command not found"
        fi
        MV="sudo mv"
    fi
    $MV "$_dfx_file" "${_install_dir}" 2>/dev/null \
        || err "Failed to install the DFINITY Development Kit: please check your permissions and try again."

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

    if [ "$_ostype" = Darwin ] && [ "$_cputype" = arm64 ]; then
        # Support for M1 Macs using Rosetta for the time being.
        # We specialize on Darwin here because we cannot support ARM natively.
        _cputype=x86_64
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
    rm -f "${uninstall_file_path}"
    printf "%s" "$uninstall_script" >"${uninstall_file_path}"
    ensure chmod u+x "${uninstall_file_path}"
}

main "$@" || exit $?
