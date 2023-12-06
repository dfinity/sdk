## 999_footer.sh

DFXVM_GITHUB_LATEST_RELEASE_ROOT="${DFXVM_GITHUB_LATEST_RELEASE_ROOT:-https://github.com/dfinity/dfxvm/releases/latest/download}"
DFX_VERSION="${DFX_VERSION-}"

# The SHA and the time of the last commit that touched this file.
SCRIPT_COMMIT_DESC="@revision@"

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
            # The only cputype we build on Linux is x86_64.
            # `uname -m` in a Linux Docker container on an Apple M1 can return aarch64
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
