#!/usr/bin/env bash

DFX_STANDALONE="$(nix-build -A dfx.standalone)"

"$DFX_STANDALONE/bin/dfx" cache install
CACHE_DIR="$("$DFX_STANDALONE/bin/dfx" cache show)"
echo "cache dir is $CACHE_DIR"

if uname -a | grep Linux; then
    LIB_LIST_TOOL="ldd"
else
    LIB_LIST_TOOL="otool -L"
fi

result=0
for a in dfx ic-ref ic-starter icx-proxy mo-doc mo-ide moc replica;
do
    echo
    echo "checking $a"

    if ! output="$($LIB_LIST_TOOL "$CACHE_DIR/$a" 2>&1)"; then
        echo "$output"
        if echo "$output" | grep -q "not a dynamic executable"; then
            continue
        else
            result=1
        fi
    else
        echo "$output"
        echo
        if matches="$(echo "$output" | grep -v '^\/' | grep "/nix/store")"; then
            echo "** fails because $a references /nix/store:"
            echo "$matches"
            result=1
        fi
    fi
done
exit $result
