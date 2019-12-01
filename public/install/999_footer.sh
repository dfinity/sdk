## 999_footer.sh

# If DFX_RELEASE_ROOT is unset or empty, default it.
SDK_WEBSITE="https://sdk.dfinity.org"
DFX_RELEASE_ROOT="${DFX_RELEASE_ROOT:-$SDK_WEBSITE/downloads/dfx}"
DFX_MANIFEST_JSON_URL="${DFX_MANIFEST_JSON_URL:-$SDK_WEBSITE/manifest.json}"
DFX_VERSION="${DFX_VERSION:-}"

# The SHA and the time of the last commit that touched this file.
SCRIPT_COMMIT_DESC="@revision@"

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
    _version="$(downloader ${DFX_MANIFEST_JSON_URL} - | get_tag_from_manifest_json latest)" || return 2

    printf %s "${_version}"
}

validate_install_dir() {
    local dir="${1%/}"

    # We test it's a directory and writeable.
    ! [ -d $dir ] && return 1
    ! [ -w $dir ] && return 2

    # We also test it's in the $PATH of the user.
    case ":$PATH:" in
        *:$dir:*) ;;
        *) return 3 ;;
    esac

    return 0
}

sdk_install_dir() {
    if [ "${DFX_INSTALL_ROOT:-}" ]; then
        # If user specifies an actual dir, use that.
        printf %s "${DFX_INSTALL_ROOT}"
    elif validate_install_dir /usr/local/bin; then
        printf %s /usr/local/bin
    elif [ "$(uname -s)" = Darwin ]; then
        # OS X does not allow users to write to /usr/bin by default. In case the
        # user is missing a /usr/local/bin we need to create it. Even if it is
        # not "writeable" we expect the user to have access to super user
        # privileges during the installation.
        mkdir -p /usr/local/bin 2>/dev/null || sudo mkdir -p /usr/local/bin || true
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
    log "Executing DFINITY SDK install script, commit: $SCRIPT_COMMIT_DESC"

    downloader --check
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd tar
    need_cmd gzip
    need_cmd touch
    # For instance in Debian sudo can be missing.
    need_cmd sudo

    if ! confirm_license; then
        echo "Please accept the license to continue."
        exit
    fi

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
    local _dfx_url="${DFX_RELEASE_ROOT}/${DFX_VERSION}/${_arch}/dfx-${DFX_VERSION}.tar.gz"

    local _dir
    _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t dfinity-sdk)"
    local _dfx_archive="${_dir}/dfx.tar.gz"
    local _dfx_file="${_dir}/dfx"

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

    mv "$_dfx_file" "${_install_dir}" 2>/dev/null || sudo mv "$_dfx_file" "${_install_dir}" \
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
    touch "${uninstall_file_path}"
    printf "%s" "$uninstall_script" >"${uninstall_file_path}"
    ensure chmod u+x "${uninstall_file_path}"
}

main "$@" || exit $?
