## 300_license.sh

confirm_license() {
    local header notice prompt
    header="
    \x1b[1mThe DFINITY Canister SDK\x1b[0m

    Please READ the following NOTICE:"
    notice='Copyright 2021 DFINITY Stiftung. All Rights Reserved.

The DFINITY Canister SDK (the \"Software\") is licensed under the Alpha DFINITY
Canister SDK License Agreement (the \"License\"). You may not use the Software
except in compliance with the License. You may obtain a copy of the License at

    https://sdk.dfinity.org/sdk-license-agreement.txt

The Software is provided to you AS IS and WITHOUT WARRANTY.'
    prompt="Do you agree and wish to install the DFINITY Canister SDK [y/N]?"

    # we test if there is a terminal present (that is, STDIN is a TTY)
    # Just -t 0 alone doesn't work for SSH connections, so test for a socket
    if ! [ -t 0 ] && ! [ -p /dev/stdin ]; then
        printf "%s\n" "Please run in an interactive terminal."
        printf "%s\n" "Hint: Run  sh -ci \"\$(curl -fsSL $SDK_WEBSITE/install.sh)\""
        exit 0
    fi
    printf "%b\n\n" "$header"
    printf "%b\n\n" "$notice"
    printf "%b\n" "$prompt"
    while true; do
        read -r resp
        case "$resp" in
            # Continue on yes or y.
            [Yy][Ee][Ss] | [Yy])
                return 0
                ;;
            # Exit on no or n, or <enter>
            [Nn][Oo] | [Nn] | '')
                return 1
                ;;
            *)
                # invalid input
                # Send out an ANSI escape code to move up and then to delete the
                # line. Keeping it separate for convenience.
                printf "%b\n" "\033[2A"
                printf "%b " "\r\033[KAnswer with a yes or no to continue. [y/N]"
                ;;
        esac
    done
}
