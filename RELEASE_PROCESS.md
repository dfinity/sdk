# Release Process

This document describes the release process for DFINITY SDK.
We review the mechanism, the automation provided and the checklist process.

# Overview

Releases are done following the checklist rule: preferrably by two people familiar with the process, i.e. 1 driver 1 validator.
A successful release is the result of coordination between automation, manual labor and validation.
We should improve the release process continuously.
In the following mechanism section we describe the operation of how to get a new release out.

# Checklist
- [ ] Participants
   - [ ] Driver
   - [ ] Validator
   - [ ] CI (system up)
- [ ] Is master green?
- [ ] Was master red recently or flaky?
- [ ] Follow process:
   1. - [ ] Update cargo version (Use `./scripts/update-cargo-version.sh version`)
   2. - [ ] Update manifest to include new version; *Ensure* latest remains the same
   3. - [ ] Switch to stable branch and `git pull stable`
   4. - [ ] Run `git pull master --ff`
   5. - [ ] Create tags with `git tag -a version -m " Release: version"`
   6. - [ ] Double check the tag points correctly and is an annotated one: `git log` and  `git describe --always`
   7. - [ ] Push tag first to avoid triggering hydra incorrectly `git push origin version`
   8. - [ ] Push the stable branch now `git push origin stable`
   9. - [ ] Check hydra
   10. - [ ] Prepare PR for manifest
   11. - [ ] Update release notes
   12. - [ ] Update manifest once all builds are done
			 - [ ] Linux
			 - [ ] Darwin



## Requirements & Properties

 - Semi-automation
 - Consistent delivery
 - Validation
 - Rollback
 - Railguards
 - Flexibity

## Mechanism
TODO


### Build
### CI
### Manifest
### Changelog


### Process

We now summarize the release process.



# TODOs and Improvements
*** Cargo version from the tag
*** Add release stress tests
*** Add valid json test for the manifest
