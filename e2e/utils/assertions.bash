#!

# Asserts that a command line succeeds. Still sets $output to the stdout and stderr
# of the command.
# Arguments:
#   $@ - The command to run.
# Returns:
#   none
assert_command() {
    local stdoutf stderrf statusf
    stderrf="$(mktemp)"
    stdoutf="$(mktemp)"
    statusf="$(mktemp)"
    ( set +e; "$@" 2>"$stderrf" >"$stdoutf"; echo -n "$?" > "$statusf" )
    status=$(< "$statusf"); rm "$statusf"
    stderr=$(< "$stderrf"); rm "$stderrf"
    stdout=$(< "$stdoutf"); rm "$stdoutf"
    output="$(
        if [ "$stderr" ]; then echo "$stderr"; fi 
        if [ "$stdout" ]; then echo "$stdout"; fi
    )"

    if [[ ! $status -eq 0 ]]; then 
        (echo "$*"; echo "status: $status"; echo "$output" | batslib_decorate "Output") \
         | batslib_decorate "Command failed" \
         | fail
    fi
}

# Asserts that a command line fails. Still sets $output to the stdout and stderr
# of the command.
# Arguments:
#   $@ - The command to run.
# Returns:
#   none
assert_command_fail() {
    local stdoutf stderrf statusf
    stderrf="$(mktemp)"
    stdoutf="$(mktemp)"
    statusf="$(mktemp)"
    ( set +e; "$@" 2>"$stderrf" >"$stdoutf"; echo -n "$?" >"$statusf" )
    status=$(< "$statusf"); rm "$statusf"
    stderr=$(< "$stderrf"); rm "$stderrf"
    stdout=$(< "$stdoutf"); rm "$stdoutf"
    output="$(
        if [ "$stderr" ]; then echo "$stderr"; fi;
        if [ "$stdout" ]; then echo "$stdout"; fi;
    )"

    if [[ $status -eq 0 ]]; then
        ( echo "$*"; echo "$output" | batslib_decorate "Output") \
            | batslib_decorate "Command succeeded (should have failed)" \
            | fail
    fi
}

# Asserts that a string contains another string, using regexp.
# Arguments:
#    $1 - The regex to use to match.
#    $2 - The string to match against (output). By default it will use
#         $output.
assert_match() {
    regex="$1"
    if [[ $# -lt 2 ]]; then
        text="$output"
    else
        text="$2"
    fi
    if [[ ! "$text" =~ $regex ]]; then
        batslib_print_kv_single_or_multi 10 "regex" "$regex" "actual" "$text" \
            | batslib_decorate "output does not match" \
            | fail
    fi
}

# Asserts that a string contains another string
# Arguments:
#    $1 - The string to search for.
#    $2 - The string to search in.
assert_contains() {
    search_for="$1"
    if [[ $# -lt 2 ]]; then
        search_in="$output"
    else
        search_in="$2"
    fi
    if [[ ! "$search_in" == *"$search_for"* ]]; then
        batslib_print_kv_single_or_multi 10 "search phrase" "$search_for" "actual output" "$search_in" \
            | batslib_decorate "output does not match" \
            | fail
    fi
}

# Asserts that a string does not contain another string, using regexp.
# Arguments:
#    $1 - The regex to use to match.
#    $2 - The string to match against (output). By default it will use
#         $output.
assert_not_match() {
    regex="$1"
    if [[ $# -lt 2 ]]; then
        text="$output"
    else
        text="$2"
    fi
    if [[ "$text" =~ $regex ]]; then
        batslib_print_kv_single_or_multi 10 "regex" "$regex" "actual" "$text" \
            | batslib_decorate "output matches but is expected not to" \
            | fail
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
    if [[ $# -lt 2 ]]; then
        actual="$output"
    else
        actual="$2"
    fi

    if [[ ! "$actual" == "$expected" ]]; then
        batslib_print_kv_single_or_multi 10 "expected" "$expected" "actual" "$actual" \
            | batslib_decorate "output does not match" \
            | fail
    fi
}

# Asserts that two values are not equal.
# Arguments:
#    $1 - The expected value.
#    $2 - The actual value.
assert_neq() {
    expected="$1"
    if [[ $# -lt 2 ]]; then
        actual="$output"
    else
        actual="$2"
    fi

    if [[ "$actual" = "$expected" ]]; then
        batslib_print_kv_single_or_multi 10 "expected" "$expected" "actual" "$actual" \
            | batslib_decorate "output does not match" \
            | fail
    fi
}


# Asserts that a process exits within a timeframe
# Arguments:
#    $1 - the PID
#    $2 - the timeout
assert_process_exits() {
    pid="$1"
    timeout="$2"

    echo "waiting up to $timeout seconds for process $pid to exit"

    timeout "$timeout" sh -c \
      "while kill -0 $pid; do echo waiting for process $pid to exit; sleep 1; done" \
      || (echo "process $pid did not exit" && ps aux && exit 1)

    echo "process $pid exited"
}

# Asserts that `dfx start` and `replica` are no longer running
assert_no_dfx_start_or_replica_processes() {
    ! ( pgrep "dfx start" )
    if [ -e .dfx/replica-configuration/replica-pid ];
    then
      ! ( kill -0 "$(< .dfx/replica-configuration/replica-pid)" 2>/dev/null )
    fi
}

assert_file_exists() {
    filename="$1"

    if [[ ! -f $filename ]]; then
        echo "$filename does not exist" \
        | batslib_decorate "Missing file" \
        | fail
    fi
}

assert_file_not_empty() {
    filename="$1"

    assert_file_exists "$filename"

    if [[ ! -s $filename ]]; then
        echo "$filename is empty" \
        | batslib_decorate "Empty file" \
        | fail
    fi
}

assert_file_empty() {
    filename="$1"

    assert_file_exists "$filename"

    if [[ -s $filename ]]; then
        echo "$filename is not empty" \
        | batslib_decorate "File not empty" \
        | fail
    fi
}

assert_file_not_exists() {
    filename="$1"

    if [[ -f $filename ]]; then
        head -n 10 "$filename" \
        | batslib_decorate "Expected file to not exist. $filename exists and starts with:" \
        | fail
    fi
}

assert_file_eventually_exists() {
    filename="$1"
    timeout="$2"

    timeout "$timeout" sh -c \
      "until [ -f \"$filename\" ]; do echo waiting for \"$filename\"; sleep 1; done" \
      || (echo "file \"$filename\" was never created" && ls && exit 1)
}

assert_directory_not_exists() {
    directory="$1"
    if [[ -d $directory ]]; then
        ( echo "Contents of $directory:" ; ls -AlR "$directory" ) \
        | batslib_decorate "Expected directory '$directory' to not exist." \
        | fail
    fi
}

# Asserts that the contents of two files are equal.
# Arguments:
#    $1 - The name of the file containing the expected value.
#    $2 - The name of the file containing the actual value.
assert_files_eq() {
    expected="$(cat "$1")"
    actual="$(cat "$2")"

    if [[ ! "$actual" == "$expected" ]]; then
        diff "$1" "$2" \
            | batslib_decorate "contents of $1 do not match contents of $2" \
            | fail
    fi
}
