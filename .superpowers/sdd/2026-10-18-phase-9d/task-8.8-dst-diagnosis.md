# Task 8.8 DST diagnosis

The seasonal failure comes from the native routine's access-time conversion before its ambiguous requested-time conversion. Fixed winter and summer access timestamps select different oracle epochs, while both epochs read back as the requested local time. Crossed access/modification timestamps and preceding-call controls isolate access time as the deciding input in these probes. Fixed initial metadata alone does not repair the full witness, because a later file copy reads the source and replaces its access timestamp with the current time.

## Contract and source evidence

The local reference, `oodocs/rexxref/en-US/rexxutil.xml:3166-3233`, specifies setting the last-modified timestamp, preserving an omitted date/time portion, accepted input formats and success/failure codes. This section gives no rule choosing either occurrence of an ambiguous local time. The native source, `/home/moritz/dev/repos/ooRexx/interpreter/platform/unix/SysRexxUtil.cpp:1415`, first calls `mktime(localtime(st_atime))` to preserve access time. It then seeds fields from `localtime(st_mtime)`, replaces supplied fields, sets `tm_isdst=-1` and calls `mktime` at lines 1426-1448.

The Rust implementation, `rust/crates/rexx-exec/src/builtin/rexxutil/files.rs:298-336`, deterministically chooses the earliest candidate that reads back correctly. Its leading comment describes this as `mktime` with `tm_isdst=-1`, but the measurements below disprove exact equivalence for ambiguous inputs on this host. `rust/crates/rexx-exec/tests/datetime_zone.rs:143-213` already supplies explicit Zurich/New York zones and compares full outputs, so pinning TZ again cannot resolve the failure. The queued seasonal observation is reproduced by varying only fixed timestamps, without waiting for the host's DST state to change.

## Execution and provenance

All diagnostics used fresh per-invocation directories under `/tmp/p9d-t88-dst-20261030-a`. The crate binary was copied before the first invocation to `rexx-frozen`, from the assignment's existing `/tmp/codex-p9d-t88-target/debug/rexx` build identified as revision `3148553fe`. Its SHA-256 is `c52dce960bacae58e9f7fff86342a64df1dc177dbbe4c80a82690633e02cf722`; copying and hashing preserve the actual tested artifact independently of subsequent shared builds. The diagnostic did not build, edit source, change the index or change HEAD.

The current oracle was `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, with its `build/lib` on `LD_LIBRARY_PATH`. Its binary SHA-256 is `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019`, and `librexx.so.4` is `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb`. Both interpreters returned `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` from `parse version`. `provenance.json` also records binary sizes and the observed repository HEADs, which are contextual provenance rather than proof of the oracle binary's source revision.

The foreground drivers ran as `memcap 8G python3 /tmp/p9d-t88-dst-driver.py`, `memcap 8G python3 /tmp/p9d-t88-dst-supplement.py` and `memcap 8G python3 /tmp/p9d-t88-dst-full-trace.py`. Each crate child used `memcap 2G timeout -k 2s 15s rexx-frozen PROGRAM`; each oracle child used the retained literal `oracle.sh`, which applies `ulimit -v 1048576`, sets the required library path and executes `timeout -k 2s 15s` around the oracle. Python also applied a 20-second subprocess deadline and captured stdout and stderr separately. All three driver commands exited 0, and no child timed out or hit a memory limit.

There were 74 interpreter invocations: 58 reduced/version probes exited 0 with empty stderr, and 16 full-witness variants exited 216 at the witness's deliberate final invalid `SysGetFileDateTime` selector. Each invocation retains separate `stdout`, `stderr`, `status` and `descriptor.json` files. Initial and final access/modification nanoseconds appear in reduced descriptors, while full-witness epoch lines and metadata traces preserve the relevant timestamps before its cleanup removes the file. The initial full runs use distinct absolute source paths, so their error text contains corresponding path differences; the shared-source metadata trace has byte-identical stderr on both sides.

## Reduced results

Winter means epoch `1577880000`, and summer means `1593604800`, for both initial timestamps unless the row says otherwise. Every row runs in a new process and directory, with the target already created and seeded by the driver. Each reduced program calls `SysSetFileDateTime` once on that target and prints the return code and `SysGetFileDateTime` result. All return codes are 0, all overlap readbacks equal the supplied local fields, and access timestamps remain unchanged.

| Zone and requested local time | Initial access time | Initial modification time | Crate epoch | Oracle epoch |
| --- | --- | --- | --- | --- |
| Zurich, 2020-10-25 02:30:00 | Winter | Winter | 1603585800 | 1603589400 |
| Zurich, 2020-10-25 02:30:00 | Summer | Summer | 1603585800 | 1603585800 |
| Zurich, 2020-10-25 02:30:00 | Winter | Summer | 1603585800 | 1603589400 |
| Zurich, 2020-10-25 02:30:00 | Summer | Winter | 1603585800 | 1603585800 |
| New York, 2020-11-01 01:30:00 | Winter | Winter | 1604208600 | 1604212200 |
| New York, 2020-11-01 01:30:00 | Summer | Summer | 1604208600 | 1604208600 |
| New York, 2020-11-01 01:30:00 | Winter | Summer | 1604208600 | 1604212200 |
| New York, 2020-11-01 01:30:00 | Summer | Winter | 1604208600 | 1604208600 |
| Dublin, 2020-10-25 01:30:00 | Winter | Winter | 1603585800 | 1603589400 |
| Dublin, 2020-10-25 01:30:00 | Summer | Summer | 1603585800 | 1603585800 |
| Dublin, 2020-10-25 01:30:00 | Winter | Summer | 1603585800 | 1603589400 |
| Dublin, 2020-10-25 01:30:00 | Summer | Winter | 1603585800 | 1603585800 |

For Zurich and New York, separate controls first set another file to either 2020-01-01 or 2020-07-01 at 12:00, then request the target overlap. Every target still follows its own winter/summer access timestamp, regardless of that preceding operation. This supports the source-derived mechanism: each target call's access-time `mktime` supersedes the preceding conversion history relevant to these examples. It does not establish a portable overlap-selection algorithm for every libc, zone or historical transition.

The spring-gap controls agree exactly for both access seeds: Zurich 2020-03-29 02:30 becomes epoch `1585445400` and reads 03:30, while New York 2020-03-08 02:30 becomes `1583652600` and reads 03:30. The ordinary 2020-06-01 12:34:56 controls also agree exactly, producing `1591007696` in Zurich and `1591029296` in New York. These controls preserve evidence that zone conversion matters and that gap behavior is a distinct contract from overlap selection. No justification emerged for changing the runtime's ordinary or gap conversion.

## Full-witness sequence

The unmodified copied full witness reproduces Zurich's queued `1603585800` versus `1603589400` mismatch. Winter seeding reproduces it independently of the date of execution, while summer seeding makes that Zurich row agree; winter seeding similarly exposes New York's `1604208600` versus `1604212200` mismatch. Both initial zones still reach a Dublin overlap mismatch of `1603585800` versus `1603589400`, even when initial access and modification metadata are fixed to summer. The remaining difference therefore requires examining later operations rather than extending the initial seed.

The shared-source trace records initial summer metadata `1593604800 1593604800`. Before the `blank, utf8` row it records access time `1000000000`, following the witness's explicit access-time reset, and immediately after that copy it records access time `1793365441` on both sides. That current access timestamp persists to the program's switch to Europe/Dublin. The source copy therefore reintroduces seasonal metadata after initialization, explaining why initial summer seeding alone cannot repair the complete witness.

## Recommended implementation boundary

The controller accepted replacing exact ambiguous-epoch output with a validity/local-roundtrip property, keeping exact ordinary and spring-gap comparisons and the distinct-output assertion across zones. Check overlap results against the two valid epochs for fixed named-zone controls, and check that each selected epoch resolves to the requested local fields. Keep the corpus's local readbacks and its exact epoch observations for unambiguous inputs, including the transition boundaries around each overlap. Do not switch `make_time` to the later candidate or reproduce libc's incidental access-time conversion history merely to force one oracle answer.

This is a fixture-contract repair, and the controller explicitly ruled that it does not introduce a numbered Deviation. Fixed initialization can still make unrelated metadata reproducible, but it must not serve as proof that exact overlap epochs are portable or stable through this witness's later file reads. Correcting the inaccurate `make_time` comment is justified if that comment is touched, with the distinction between a deterministic overlap policy and host `mktime` behavior stated plainly. Implementation, sourceline fixture regeneration and the existing `datetime_zone`/corpus gates remain the implementer's work; this report makes no completion claim about Task 8.8.

## Raw evidence inventory

* `/tmp/p9d-t88-dst-driver.py`: initial 62-invocation driver, including reduced controls and unmodified/winter/summer full-witness copies.
* `/tmp/p9d-t88-dst-supplement.py`: eight Dublin crossed-metadata controls.
* `/tmp/p9d-t88-dst-full-trace.py`: four shared-source full-witness metadata traces.
* `/tmp/p9d-t88-dst-20261030-a/provenance.json`: versions' artifact provenance, sizes and hashes.
* `/tmp/p9d-t88-dst-20261030-a/results.json`, `dublin-results.json`, `full-metadata-results.json` and `summary.json`: complete invocation descriptors and status totals.
* `/tmp/p9d-t88-dst-20261030-a/{zurich,new_york}-overlap-*-{crate,oracle}/`: reduced sources, initial/final epochs and independent descriptors.
* `/tmp/p9d-t88-dst-20261030-a/dublin-overlap-*-{crate,oracle}/`: Dublin controls and descriptors.
* `/tmp/p9d-t88-dst-20261030-a/{zurich,new_york}-full-{current,winter,summer}-{crate,oracle}/`: copied full sources and independent descriptors.
* `/tmp/p9d-t88-dst-20261030-a/full-metadata-trace.rex` and `{zurich,new_york}-full-metadata-{crate,oracle}/`: shared instrumented source, timestamp traces and independent descriptors.
* `/tmp/p9d-t88-dst-20261030-a/*full*.diff`: retained stdout differences for all full-witness variants.
