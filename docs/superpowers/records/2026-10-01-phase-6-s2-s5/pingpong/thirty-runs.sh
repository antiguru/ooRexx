#!/bin/bash
# 30 oracle runs of each ping-pong program from a fresh empty dir: distinct
# stdout, stderr and rc.
B=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/bench-programs/pingpong
for p in pingmsg pingsem pingguard; do
    d=$(mktemp -d /tmp/claude-1000/p6-t23/o30.XXXX)
    for i in $(seq 30); do
        ( cd "$d" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $B/$p.rex 2>&1; echo "rc=$?" )
    done | sort | uniq -c | sed "s/^/$p /"
    rmdir "$d"
    for i in $(seq 30); do
        ( timeout -s KILL 20 /tmp/claude-1000/p6-t23/target/release/rexx-run $B/$p.rex 2>&1; echo "rc=$?" )
    done | sort | uniq -c | sed "s/^/$p rexx-run /"
done
