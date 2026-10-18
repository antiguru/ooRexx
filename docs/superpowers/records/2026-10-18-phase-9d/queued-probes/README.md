# Queued-item probes

Each directory holds one probe `<name>.rex`, its standard input as `stdin.txt` where it has one,
and each side's `oracle.*` and `crate.*` stdout, stderr and rc. The table citing them is
`../queued-inventory.tsv`.

Each side ran once, on 2026-10-18 at `be41dcc1c`, from its own fresh empty directory with the
program at `<name>/<name>.rex`:

* oracle: `( cd $DIR && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
  timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx <name>/<name>.rex <stdin )`;
* crate: `( cd $DIR && timeout -k 5 20 memcap 2G $T/release/rexx <name>/<name>.rex <stdin )`, with
  `$T` built by `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx`.

The run directory's absolute path is replaced by `RUNDIR` in stdout and stderr. `frames1` and
`frames2` had their `.cls` files beside the program, and `exttr` had `ext.rex` there.
`stdoutclosed` ran by hand with standard output closed (`>&-`), stderr and rc recorded.
`memcompile/rss.txt` holds the maximum resident set sizes and their commands.

`bgcmd` is not a queued item. It is the first form of `fifo`, whose background writer kept the
command's standard output open: the crate waits on it until the timeout (rc 124) and the oracle
does not.
