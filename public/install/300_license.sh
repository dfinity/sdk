## 300_license.sh

confirm_license() {
    local prompt header license
    header="\n DFINITY SDK \n Please READ the following license: \n\n"

    license="DFINITY Foundation -- All rights reserved. This is an ALPHA version
of the DFINITY Canister Software Development Kit (SDK). Permission is hereby granted
to use AS IS and only subject to the Alpha DFINITY Canister SDK License Agreement which
can be found here [https://sdk.dfinity.org/sdk-license-agreement.txt]. It comes with NO WARRANTY.\n"

    prompt='Do you agree and wish to install the DFINITY ALPHA SDK [y/N]?'

    # we test if there is a terminal present (that is, STDIN is a TTY)
    # Just -t 0 alone doesn't work for SSH connections, so test for a socket
    if ! [ -t 0 ] && ! [ -p /dev/stdin ]; then
        printf "%s\n" "Please run in an interactive terminal."
        printf "%s\n" "Hint: Run  sh -ci \"\$(curl -fsSL $SDK_WEBSITE/install.sh)\""
        exit 0
    fi
    printf "%b" "$header"
    printf "%b\n\n" "$license"
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
