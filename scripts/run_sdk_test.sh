#!/usr/bin/env bash

# This script is called from a GitLab pipeline.
# See .gitlab-ci.yml in the project root directory.
# Runs e2e sdk tests against a given upstream git branch and commit.

set -euo pipefail

echo "Running SDK tests against $SDK_TEST_BRANCH_NAME $SDK_TEST_COMMIT_SHA"

if [ -z "$SDK_VER" ]; then
  export SDK_VER=$(jq -r '.tags.latest' public/manifest.json)
fi

git switch --detach $SDK_VER
contents=$(jq --indent 4 ".dfinity.ref = \"$SDK_TEST_BRANCH_NAME\" | .dfinity.rev = \"$SDK_TEST_COMMIT_SHA\"" nix/sources.json) && echo "$contents" > nix/sources.json
nix-build --max-jobs 10 -A e2e-tests . -o $CI_JOB_STAGE/$CI_JOB_NAME --show-trace
