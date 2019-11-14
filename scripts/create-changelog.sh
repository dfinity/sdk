#!/usr/bin/env sh


set -u

retrieve_changelog() {
    # we remove 40chars+space
    git log $1..$2 --pretty=oneline | grep -e 'feat:' -e 'fix:' | colrm 1 41
}

retrieve_changelog $@
