#!/usr/bin/env bash

set -e

DFX_STANDALONE="$(nix-build -A dfx.standalone)"

"$DFX_STANDALONE/bin/dfx" cache install
CACHE_DIR="$("$DFX_STANDALONE/bin/dfx" cache show)"
echo "cache dir is $CACHE_DIR"

if uname -a | grep Linux; then
    LIB_LIST_TOOL="ldd"
else
    LIB_LIST_TOOL="otool -L"
fi

check_binary() {
    path=$1
    echo
    echo "checking $path"

    if ! output="$($LIB_LIST_TOOL "$path" 2>&1)"; then
        echo "$output"
        if echo "$output" | grep -q "not a dynamic executable"; then
            return 0
        else
            return 1
        fi
    else
        echo "$output"
        echo
        if found="$(echo "$output" | grep -v '^\/' | grep "/nix/store")"; then
            echo "** fails because $path references /nix/store:"
            echo "$found"
            return 1
        else
            return 0
        fi
    fi
}

result=0

if ! check_binary "$DFX_STANDALONE/bin/dfx"; then
  result=1
fi

for a in ic-ref ic-starter icx-proxy mo-doc mo-ide moc replica;
do
  if ! check_binary "$CACHE_DIR/$a"; then
      result=1
  fi
done
exit $result
