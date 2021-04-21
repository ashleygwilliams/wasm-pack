#!/bin/bash
# Copyright 2016 The Rust Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution and at
# http://rust-lang.org/COPYRIGHT.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# This is just a little script that can be downloaded from the internet to
# install wasm-pack. It just does platform detection, downloads the installer
# and runs it.

set -u

UPDATE_ROOT="https://github.com/rustwasm/wasm-pack/releases/download/$VERSION"

main() {
    downloader --check
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd rmdir
    need_cmd tar
    need_cmd which
    need_cmd dirname

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    local _ext=""
    case "$_arch" in
        *windows*)
            _ext=".exe"
            ;;
    esac

    which rustup > /dev/null 2>&1
    need_ok "failed to find Rust installation, is rustup installed?"
    local _rustup=`which rustup`
    local _tardir="wasm-pack-$VERSION-${_arch}"
    local _url="$UPDATE_ROOT/${_tardir}.tar.gz"
    local _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t wasm-pack)"
    local _file="$_dir/input.tar.gz"
    local _wasmpack="$_dir/wasm-pack$_ext"
    local _wasmpackinit="$_dir/wasm-pack-init$_ext"

    printf '%s\n' 'info: downloading wasm-pack' 1>&2

    ensure mkdir -p "$_dir"
    downloader "$_url" "$_file"
    if [ $? != 0 ]; then
      say "failed to download $_url"
      say "this may be a standard network error, but it may also indicate"
      say "that wasm-pack's release process is not working. When in doubt"
      say "please feel free to open an issue!"
      exit 1
    fi
    ensure tar xf "$_file" --strip-components 1 -C "$_dir"
    mv "$_wasmpack" "$_wasmpackinit"

    # The installer may want to ask for confirmation on stdin for various
    # operations. We were piped through `sh` though so we probably don't have
    # access to a tty naturally. If it looks like we're attached to a terminal
    # (`-t 1`) then pass the tty down to the installer explicitly.
    if [ -t 1 ]; then
      "$_wasmpackinit" "$@" < /dev/tty
    else
      "$_wasmpackinit" "$@"
    fi

    local _retval=$?

    ignore rm -rf "$_dir"

    return "$_retval"
}

get_architecture() {
    local _ostype="$(uname -s)"
    local _cputype="$(uname -m)"

    if [ "$_ostype" = Darwin -a "$_cputype" = i386 ]; then
        # Darwin `uname -s` lies
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            local _cputype=x86_64
        fi
    fi

    case "$_ostype" in
        Linux)
            local _ostype=unknown-linux-musl
            ;;

        Darwin)
            local _ostype=apple-darwin
            # override for case where target machine is using Apple Silicon CPU, 
            # use x86_64 binary via rosetta
            if [ $_cputype == arm64 ]; then
                local _cputype=x86_64
            fi
            ;;

        MINGW* | MSYS* | CYGWIN*)
            local _ostype=pc-windows-msvc
            ;;

        *)
            err "no precompiled binaries available for OS: $_ostype"
            ;;
    esac

    case "$_cputype" in
        x86_64 | x86-64 | x64 | amd64)
            local _cputype=x86_64
            ;;
        *)
            err "no precompiled binaries available for CPU architecture: $_cputype"

    esac

    local _arch="$_cputype-$_ostype"

    RETVAL="$_arch"
}

say() {
    echo "wasm-pack-init: $1"
}

err() {
    say "$1" >&2
    exit 1
}

need_cmd() {
    if ! check_cmd "$1"
    then err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
    return $?
}

need_ok() {
    if [ $? != 0 ]; then err "$1"; fi
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    "$@"
    need_ok "command failed: $*"
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    "$@"
}

# This wraps curl or wget. Try curl first, if not installed,
# use wget instead.
downloader() {
    if check_cmd curl
    then _dld=curl
    elif check_cmd wget
    then _dld=wget
    else _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]
    then need_cmd "$_dld"
    elif [ "$_dld" = curl ]
    then curl -sSfL "$1" -o "$2"
    elif [ "$_dld" = wget ]
    then wget "$1" -O "$2"
    else err "Unknown downloader"   # should not reach here
    fi
}

main "$@" || exit 1
