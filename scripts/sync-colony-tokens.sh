#!/usr/bin/env bash
# Refresh the Colony theme data the interface ships, from the generated output
# of Project-Colony-Resources at one commit:
#
#   scripts/sync-colony-tokens.sh <commit>
#
# The files are never edited by hand. They are copied as generated with one
# exception: em and en dashes (in the generated headers and some theme notes)
# become plain hyphens, as nothing in this repository may contain them. The
# commit they came from is recorded in src/ui/vendor/colony/SOURCE.
set -euo pipefail

commit="${1:?usage: $0 <Project-Colony-Resources commit>}"
base="https://raw.githubusercontent.com/Project-Colony/Project-Colony-Resources/${commit}/generated"
dest="$(dirname "$0")/../src/ui/vendor/colony"

# Everything is downloaded first: a failed download leaves the vendored files
# and SOURCE exactly as they were.
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for file in css/colony-themes.css css/colony-accents.css themes.json; do
  curl --fail --silent --show-error --location "$base/$file" |
    perl -CS -pe 's/[\x{2013}\x{2014}]/-/g' >"$tmp/$(basename "$file")"
done
printf 'Project-Colony-Resources %s, generated/, with em and en dashes replaced by hyphens\n' \
  "$commit" >"$tmp/SOURCE"

mkdir -p "$dest"
mv "$tmp/colony-themes.css" "$tmp/colony-accents.css" "$tmp/themes.json" "$tmp/SOURCE" "$dest/"
