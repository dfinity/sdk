## 020_flags.sh

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
    while [ -n "$@" ]; do
        local ARG=$1
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
