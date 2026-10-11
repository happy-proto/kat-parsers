#!/usr/bin/env bash
set -euo pipefail

channel=${1:?usage: download-ghostty.sh stable|tip destination}
destination=${2:?usage: download-ghostty.sh stable|tip destination}
mkdir -p "$destination"
destination=$(cd "$destination" && pwd)

case "$channel" in
  stable)
    # Stable releases are published on Ghostty's own download service, not
    # GitHub Releases. Read the currently published version from its website.
    curl --fail --location --retry 3 --silent --show-error \
      https://ghostty.org/download --output "$destination/download.html"
    urls=$(grep -Eo 'https://release.files.ghostty.org/[0-9]+\.[0-9]+\.[0-9]+/Ghostty.dmg' "$destination/download.html" | sort -u)
    test "$(printf '%s\n' "$urls" | wc -l | tr -d ' ')" = 1
    url="${urls%Ghostty.dmg}ghostty-macos-universal.zip"
    ;;
  tip)
    url=https://github.com/ghostty-org/ghostty/releases/download/tip/ghostty-macos-universal.zip
    ;;
  *) printf 'unknown channel: %s\n' "$channel" >&2; exit 1 ;;
esac

printf 'Ghostty %s download: %s\n' "$channel" "$url" >&2
curl --fail --location --retry 3 --silent --show-error "$url" --output "$destination/ghostty.zip"
shasum -a 256 "$destination/ghostty.zip" >&2
ditto -x -k "$destination/ghostty.zip" "$destination/app"
binary="$destination/app/Ghostty.app/Contents/MacOS/ghostty"
test -x "$binary"
"$binary" +version >&2
printf '%s\n' "$binary"
