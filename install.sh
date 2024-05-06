#!/usr/bin/env sh
## 000_header.sh
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
## install/010_manifest.sh
get_tag_from_manifest_json() {
    cat \
        | tr -d '\n' \
        | grep -o "\"$1\":[[:space:]]*\"[a-zA-Z0-9.]*" \
        | grep -o "[0-9.]*$"
}
## 020_flags.sh
DFX_BOOL_FLAGS=""
define_flag_BOOL() {
    local VARNAME
    VARNAME="flag_$(echo "$1" | tr /a-z/ /A-Z)"
    eval "$VARNAME=\${$VARNAME:-}"
    DFX_BOOL_FLAGS="${DFX_BOOL_FLAGS}--${1} $VARNAME $2"
}
get_flag_name() {
    echo "$1"
}
get_var_name() {
    echo "$2"
}
read_flags() {
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
## 100_log.sh
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
## 110_assert.sh
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
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}
ignore() {
    "$@"
}
## 200_downloader.sh
define_flag_BOOL "insecure" "Allows downloading from insecure URLs, either using HTTP or TLS 1.2 or less."
check_help_for() {
    local _cmd
    local _arg
    local _ok
    _cmd="$1"
    _ok="y"
    shift
    if check_cmd sw_vers; then
        case "$(sw_vers -productVersion)" in
            10.15*) ;;
            11.*) ;;
            12.*) ;;
            13.*) ;;
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
check_support_for() {
    local err="$1"
    shift
    local cmd="$*"
    ! ($cmd 2>&1 | grep "$err" >/dev/null)
}
downloader() {
    local _dld
    if check_cmd curl; then
        _dld=curl
    elif check_cmd wget; then
        _dld=wget
    else
        _dld='curl or wget'
    fi
    if [ "$1" = --check ]; then
        need_cmd "$_dld"
    elif [ "$_dld" = curl ]; then
        if check_help_for curl --proto --tlsv1.2; then
            curl --proto '=https' --tlsv1.2 --show-error --fail --connect-timeout 10 --retry 5 --location "$1" --output "$2" --progress-bar
        elif ! [ "$flag_INSECURE" ]; then
            warn "Not forcing TLS v1.2, this is potentially less secure"
            curl --show-error --fail --connect-timeout 10 --retry 5 --location "$1" --output "$2" --progress-bar
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
        err "Unknown downloader"
    fi
}
## 999_footer.sh
DFXVM_GITHUB_LATEST_RELEASE_ROOT="${DFXVM_GITHUB_LATEST_RELEASE_ROOT:-https://github.com/dfinity/dfxvm/releases/latest/download}"
DFX_VERSION="${DFX_VERSION-}"
SCRIPT_COMMIT_DESC="f887039ff0618f2d7e1416773561d5b68ec21352"
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
    ensure downloader "$_tarball_url" "${_tarball_filename}"
    ensure downloader "$_sha256_url" "${_sha256_filename}"
    log "Checking integrity of tarball..."
    ensure "$SHASUM" -c "${_sha256_filename}"
    ensure tar -xzf "${_tarball_filename}"
    ensure cd "${_archive}" >/dev/null
    ensure chmod u+x dfxvm
    ensure mv dfxvm dfxvm-init
    if [ -n "${DFX_VERSION}" ]; then
        ./dfxvm-init --dfx-version "${DFX_VERSION}"
    else
        ./dfxvm-init
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
    ignore rm -rf "${_dir}"
    exit $_subshell_exit_code
}
## output is one of the following, which correspond to part of the release asset filenames:
##    aarch64-apple-darwin
##    x86_64-apple-darwin
##    x86_64-unknown-linux-gnu
get_architecture() {
    local _ostype _cputype _arch
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"
    if [ "$_ostype" = Darwin ] && [ "$_cputype" = i386 ]; then
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
            _cputype=x86_64
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
