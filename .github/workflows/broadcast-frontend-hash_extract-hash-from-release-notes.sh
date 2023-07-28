#!/usr/bin/env bash

DFX_VERSION="$1" # e.g. 0.14.1

if [[ "$DFX_VERSION" == "latest" ]]
then
    gh release view --json body --jq .body > release.md
else
    gh release view "$DFX_VERSION" --json body --jq .body > release.md
fi

start=$(awk '/### Frontend canister/{ print NR; exit }' release.md)
end=$(awk -v start="$start" 'NR > start && /^### /{ print NR; exit }' release.md)

if [[ -z "$end" ]]
then
  end=$(wc -l < release.md)

fi

# subtract 1 from end, so it won't include next header line
end=$((end-1))

awk -v start="$start" -v end="$end" 'NR>=start && NR<end' release.md | awk -F": " '/Module hash/{print $2}' | tr -d $'\r'
