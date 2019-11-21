# Release Process

This document describes the release process for DFINITY SDK.
We review the mechanism, the automation provided and the checklist process.

# Checklist


# Overview

## Mechanism


## Process / Steps
1. Update or automatically update cargo
2. update manifest to include new version -- latest is the same
3. fetch origin
4. pull stable
5. pull master --ff on stable
6. git tag -a version -m "Release version"
7. git log / git describe --always
8. git push origin version
9. git push origin stable
10. check hydra
11. prep manifest pr
12. update releaset notes
13. update manifest once all darwin build

# TODOS!!!
*** Cargo version from the tag
*** manifest??? think
*** install.sh version argument
