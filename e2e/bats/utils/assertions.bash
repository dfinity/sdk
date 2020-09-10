#!

# Asserts that a command line succeeds. Still sets $output to the stdout and stderr
# of the command.
# Arguments:
#   $@ - The command to run.
# Returns:
#   none
assert_command() {
    local stderrf="$(mktemp)"
    local stdoutf="$(mktemp)"
    local statusf="$(mktemp)"
    ( set +e; "$@" 2>"$stderrf" >"$stdoutf"; echo -n "$?" > "$statusf" )
    status="$(<$statusf)"; rm "$statusf"

    stderr="$(cat $stderrf)"; rm "$stderrf"
    stdout="$(cat $stdoutf)"; rm "$stdoutf"
    output="$( \
        [ "$stderr" ] && echo $stderr || true; \
        [ "$stdout" ] && echo $stdout || true; \
    )"

    [[ $status == 0 ]] || \
        (  (echo "$*"; echo "status: $status"; echo "$output" | batslib_decorate "Output") \
         | batslib_decorate "Command failed" \
         | fail)
}

# Asserts that a command line fails. Still sets $output to the stdout and stderr
# of the command.
# Arguments:
#   $@ - The command to run.
# Returns:
#   none
assert_command_fail() {
    local stderrf="$(mktemp)"
    local stdoutf="$(mktemp)"
    local statusf="$(mktemp)"
    ( set +e; "$@" 2>"$stderrf" >"$stdoutf"; echo -n "$?" >"$statusf" )
    status="$(<$statusf)"; rm "$statusf"

    stderr="$(cat $stderrf)"; rm "$stderrf"
    stdout="$(cat $stdoutf)"; rm "$stdoutf"
    output="$(
        [ "$stderr" ] && echo $stderr || true;
        [ "$stdout" ] && echo $stdout || true;
    )"

    [[ $status != 0 ]] || \
        (  (echo "$*"; echo "$output" | batslib_decorate "Output") \
         | batslib_decorate "Command succeeded (should have failed)" \
         | fail)
}

# Asserts that a string contains another string, using regexp.
# Arguments:
#    $1 - The regex to use to match.
#    $2 - The string to match against (output). By default it will use
#         $output.
assert_match() {
    regex="$1"
    if [[ $# < 2 ]]; then
        text="$output"
    else
        text="$2"
    fi
    [[ "$text" =~ $regex ]] || \
        (batslib_print_kv_single_or_multi 10 "regex" "$regex" "actual" "$text" \
         | batslib_decorate "output does not match" \
         | fail)
}

# Asserts that a string does not contain another string, using regexp.
# Arguments:
#    $1 - The regex to use to match.
#    $2 - The string to match against (output). By default it will use
#         $output.
assert_not_match() {
    regex="$1"
    if [[ $# < 2 ]]; then
        text="$output"
    else
        text="$2"
    fi
    if [[ "$text" =~ $regex ]]; then
        (batslib_print_kv_single_or_multi 10 "regex" "$regex" "actual" "$text" \
         | batslib_decorate "output matches but is expected not to" \
         | fail)
    fi
}

# Asserts a command will timeout. This assertion will fail if the command finishes before
# the timeout period. If the command fails, it will also fail.
# Arguments:
#   $1 - The amount of time (in seconds) to wait for.
#   $@ - The command to run.

# Asserts that two values are equal.
# Arguments:
#    $1 - The expected value.
#    $2 - The actual value.
assert_eq() {
    expected="$1"
    if [[ $# < 2 ]]; then
        actual="$output"
    else
        actual="$2"
    fi

    [[ "$actual" == "$expected" ]] || \
        (batslib_print_kv_single_or_multi 10 "expected" "$expected" "actual" "$actual" \
         | batslib_decorate "output does not match" \
         | fail)
}

# Asserts that two values are not equal.
# Arguments:
#    $1 - The expected value.
#    $2 - The actual value.
assert_neq() {
    expected="$1"
    if [[ $# < 2 ]]; then
        actual="$output"
    else
        actual="$2"
    fi

    [[ "$actual" != "$expected" ]] || \
        (batslib_print_kv_single_or_multi 10 "expected" "$expected" "actual" "$actual" \
         | batslib_decorate "output does not match" \
         | fail)
}
