#!/bin/bash
# usage: crx.sh DIR FILE -- runs the crate's rexx-run (release) on DIR/FILE from DIR
cd "$1" || exit 99
memcap 4G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run "$2" >"$1/c.out" 2>"$1/c.err" </dev/null
echo "rc=$?"
echo "--- stdout"; cat -v "$1/c.out"; echo "--- stderr"; cat -v "$1/c.err"
