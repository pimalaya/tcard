#!/bin/sh

set -eu

die() {
    printf '%s\n' "$1" >&2
    exit "${2-1}"
}

DESTDIR="${DESTDIR:-}"
PREFIX="${PREFIX:-"$DESTDIR/usr/local"}"
RELEASES_URL="https://github.com/pimalaya/tcard/releases"

binary=tcard
system=$(uname -s | tr [:upper:] [:lower:])
machine=$(uname -m | tr [:upper:] [:lower:])

case $system in
    msys*|mingw*|cygwin*|win*)
	target=x86_64-windows
	binary=tcard.exe;;

    linux|freebsd)
	case $machine in
	    x86_64) target=x86_64-linux;;
	    x86|i386|i686) target=i686-linux;;
	    arm64|aarch64) target=aarch64-linux;;
	    armv6l) target=armv6l-linux;;
	    armv7l) target=armv7l-linux;;
	    *) die "Unsupported machine $machine for system $system";;
	esac;;

    darwin)
	case $machine in
	    x86_64) target=x86_64-darwin;;
	    arm64|aarch64) target=aarch64-darwin;;
	    *) die "Unsupported machine $machine for system $system";;
	esac;;

    *)
	die "Unsupported system $system";;
esac

tmpdir=$(mktemp -d) || die "Cannot create temporary directory"
trap "rm -rf $tmpdir" EXIT

echo "Downloading latest $system release…"
curl -sLo "$tmpdir/tcard.tgz" \
     "$RELEASES_URL/latest/download/tcard.$target.tgz"

echo "Installing binary…"
tar -xzf "$tmpdir/tcard.tgz" -C "$tmpdir"

mkdir -p "$PREFIX/bin"
cp -f -- "$tmpdir/$binary" "$PREFIX/bin/$binary"

die "$("$PREFIX/bin/$binary" --version) installed!" 0
