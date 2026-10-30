# Task 8.8 final review fixes

## Scope and source

This wave addresses all three Important findings in `t8.8/final-review/review.md` from verified BASE `4fe3e24d640702cddaaca25b87efc93012c1928f`. The final runtime source is `2f774eab16567dbc0a3bb758f3e29ee9bade918e`, with seven changed runtime files and no dependency, unsafe, scheduler, global root API or identity-map addition. `source-proof.json` verifies the pure moves and preservation of the earlier removal, ALLAT and SUPPLIER function bodies. Existing public exports remain available, and the moves relocate no tests or doctest blocks. Controller-owned scoped review, all-program measurement and broad final gates remain separate from this correctness handoff.

| Commit | Change |
| --- | --- |
| `f7d6e8d5e` | Pure move of stream query responsibility, preserving comments and entry points. |
| `cf3274fbf`, `abc3f2164` | Initial committed F1/F3 regressions and descriptor harness reuse. |
| `ae80e6534` | Closed POSITION handling and initial live HASITEM continuation. |
| `48bf23e06`, `9d6ce11ce`, `af734b62c` | Physical-output controls, corrected observer, and registered descriptor expectations. |
| `21af23d3f` | Pure move of OpenFile and STREAM_BUFFER from the unwieldy core body module. |
| `120c6b3f7`, `d21f4527f` | Stdout alias buffering, metadata fallback, observer-name correction and adjacent HASITEM controls. |
| `67b4e14c8`, `8e7f3ed47`, `94c8050a6` | Growth, mixed EMPTY/growth and snapshot-isolation controls, with measured oracle counts. |
| `d7091492c` | Pure move of EMPTY into the existing hash removal module. |
| `2f774eab1` | In-place EMPTY and rooted operation-selected HASITEM contents. |

## F1: Unopened and closed stdout positions

`query_position` validates every option before checking the unopened-standard state. A closed `.stdout` and unopened or closed `STDOUT`/`STDOUT:` aliases now answer the empty string for POSITION, WRITE CHAR and SYS, while opened controls retain `1`, `1` and `0` on an empty redirected file. The invalid WRITE/READ combination still raises 93.0 before the unopened result. This follows `oodocs/rexxref/en-US/streamclasses.xml:1382` and the measured `stdout-closed` oracle channels. The existing captured Invocation negative control remains independent of the host stdout descriptor.

## F2: Buffered stdout alias positions

Owned persistent nonlocal stdout aliases reuse `StreamState.open` and OpenFile's buffer and positions through a safe descriptor duplicate. A private buffering enum replaces the existing bool without adding a field, and the compiled `size_of::<StreamState>() <= 232` assertion remains unchanged. Local stdout, captured invocations, HANDLE streams and transient stdout retain their existing sink paths. Standard aliases retain their logical positions and buffered cursor across close/reopen, while status still gates unopened POSITION and descriptor-backed queries. Their owned duplicate remains part of that receiver's OpenFile until uninitialization or destruction.

The existing OpenFile buffer now distinguishes the first standard buffered write from later flushes, preserving the shared-descriptor behavior instead of adding two independent buffer lengths. `SysFile::write` initially seeks to its cached read position, then flushes at the descriptor's current shared offset, while `StreamInfo::writeBuffer` updates only that stream's logical write position. `SysFile::getPosition` includes buffered bytes without a flush, so SYS=3 with held `abc` coexists with physical size zero. SIZE flushes the alias and measures the file, following `SysFile::getSize` and the existing query-size OpenFile path. Pending local output is handed over before the alias touches the descriptor, preserving the measured ordering before and after its first buffered write.

Seven owned-descriptor fixtures cover both alias spellings, default/read/write queries, SYS, SIZE, NOBUFFER, explicit reopen and local-output interleaving. The physical observer queries an ordinary named file, `captured-output.bin`, so it neither addresses a reserved standard-stream name nor launches a command that hands over output. With local `uv` before the alias's first buffered `abc`, the final bytes are `abcD`; reversing those writes yields `uvabcD`. A NOBUFFER-to-buffered reopen retains the prior logical position before its next write while resetting the buffer mode, yielding the measured `Dvabc` and final descriptor offset 1. The harness checks stdout bytes, stderr query reports, process status, physical length and final descriptor offset separately.

## F3: HASITEM callbacks and contents lifetime

HASITEM retains the contents selected after its single HASHCODE computation and follows that contents' live links after callbacks. The three backing arrays are rooted before equality can allocate or collect, because the operation deliberately reads them after growth detaches them from the collection receiver. EMPTY now clears those arrays in place using `write_slot`, `write_link` and `set_free`, matching `HashContents::empty` and preserving write and generation invalidation. This eliminates the unreliable distinction between EMPTY and growth based only on final geometry. The traversal validates that the selected location is still occupied before reading its item or next link, and never continues through a captured entry vector or a freed location.

The initial geometry-restart implementation passed EMPTY-only controls but sent too many observable equality messages after growth. In both the growth and mixed EMPTY-then-growth probes, the oracle performs one HASHCODE and 18 equality comparisons, reports true and leaves 41 current items. The intermediate implementation performed 42 comparisons by restarting into the expanded table, while Q performed one comparison and incorrectly reported false. The final implementation follows the retained active contents and reproduces the 18-comparison result under normal execution and collection on every allocation. These exact channels supersede the earlier general description of discarding every replaced store and require no semantic licence.

Bag HASITEM shares the corrected lifetime handling while preserving its distinct protocol. Its optional index is compared against the wanted value before HASHCODE, and a successful lookup performs only the index comparison, as `BagClass::hasItemRexx` specifies. The replacement witness now sends one equality rather than Q's two, and the optional-argument witness records `equal allowed`, HASHCODE, `equal held`, then rejects `blocked` before another hash. Existing raised-send, collision, removal, ALLAT and SUPPLIER fixtures remain in the semantic suite. Bag REMOVEITEM is unchanged.

Copy and supplier isolation are supported by existing storage ownership, not by a new map or snapshot framework. `object_protocol::duplicate_collection_stores` duplicates the HASHINDEXES, HASHITEMS and HASHNEXT arrays under the Table scope, while supplier construction makes independent arrays from rooted pairs. The additional control clears and reuses an original, then clears its copy, while a previously created supplier still returns the two original values. Table, IdentityTable, Relation, Bag, Directory and StringTable all agree with the oracle and Q on that control, including actual allocation-stress execution. Earlier removal, ALLAT and indexed SUPPLIER bodies remain byte-identical to BASE, as recorded by the source proof.

## Verification and retained failures

Commands use `memcap 8G python3 /tmp/t88-final-fix-check.py NAME BOUND CWD COMMAND...`, with each exact invocation, environment, stdout, stderr, status and elapsed time retained under `final-review/fix/checks/`. The driver sets `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target` and Cargo uses `-j4`, with every foreground bound at most 520 seconds. Runtime source was committed and frozen before every compile or test. The semantic harness runs all 41 fixtures normally and with actual collection on every allocation, requiring positive collection counts. There are two added test functions in the existing semantic target and no removed or renamed historical tests or targets.

| Record | Command after the wrapper | Result |
| --- | --- | --- |
| `pure-move-policy` | `cargo test -j4 -p rexx-exec --test refusal_sites --test representation_refusals --test owners --test coverage` | 38 pass, status 0, 48.40 s. |
| `red-semantic` | `cargo test -j4 -p rexx-exec --test semantic_boundaries` | Status 101, 1.22 s; F1 and F3 fail, two existing controls pass. |
| `f1-f3-semantic` | Same semantic command | 4 pass, status 0, 39.25 s. |
| `red-alias-tests` | Semantic target filtered to `owned_stdout_aliases_keep_their_buffered_positions` | Status 101, 0.77 s; old alias WRITE/SYS values fail. |
| `green-semantic` | Full semantic target | Status 101, 38.40 s; reserved observer filename made physical SIZE fields empty. |
| `green-focused` | `cargo test -j4 -p rexx-exec --test semantic_boundaries --test standard_stream_reopen --test stream_handle --test stream_model` | 46 pass, status 0, 41.70 s. |
| `hash-stream-gc` | `cargo test -j4 -p rexx-exec --lib --test collect_stress -- hash stream` | 13 library and 1 GC test pass, status 0, 49.57 s. |
| `red-growth-semantic` | Semantic target filtered to `conversion_and_trace_boundaries` | Status 101, 1.67 s; 42 comparisons instead of 18. |
| `active-contents-semantic` | Full semantic target after the final storage correction | 5 pass, status 0, 42.71 s; all 41 normal/stress fixtures and descriptor cases. |
| `hash-copy-lib` | `cargo test -j4 -p rexx-exec --lib -- hash copy collection` | 37 pass, status 0, 60.05 s. |
| `final-policies` | `cargo test -j4 -p rexx-exec --test collect_policy --test coverage --test owners --test refusal_sites --test representation_refusals` | 40 pass, status 0, 4.37 s; no regeneration or allow-list edit. |
| `final-format` | `cargo fmt --all --check` | Status 0, 2.77 s. |
| `lint` | `cargo clippy -j4 -p rexx-core -p rexx-exec --all-targets -- -D warnings` | Status 0, 10.24 s. |
| `release` | `cargo build -j4 --release -p rexx-exec --bin rexx` | Status 0, 96.78 s. |
| `final-probes` | `python3 /tmp/t88-final-fix-probes.py final /tmp/codex-p9d-t88-pins/8.8-final-fix-rexx` | 20 probes, 60 exact oracle/crate channel agreements, every status 0; driver status 0, 4.07 s. |

One discarded observer attempt was infrastructure-only and supports no semantic conclusion. Its `stat/od /proc/self/fd/1 >&2` command redirected the descriptor before inspection, allowing the dump to read its own growing stderr file. The bounded run ended before exact-process cleanup found a remaining live target, and authorized cleanup removed four self-feeding files totaling 4,613,201,684 bytes after retaining SHA256 hashes, byte counts and 4 KiB head/tail samples. `infrastructure-cleanup.json` names every removed path, and the failed source, commands, statuses and remaining bounded records are preserved. No raw performance profile, build cache or semantic-probe channel outside that invalid attempt was removed.

## Pin and handoff

The immutable pin is `/tmp/codex-p9d-t88-pins/8.8-final-fix-rexx`, SHA256 `1e528101880f326e816e85715fb8c5401e45c62382a7b172dd113c050206ff6d`, built from `2f774eab16567dbc0a3bb758f3e29ee9bade918e`. `release-pin.json` records compiler `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 23.1.1, build/source hashes and equality of the build and pin executable hashes. The final probe driver runs each oracle process in a fresh controlled directory with the documented library path, 1 GiB address-space limit and `timeout -k 5 20`, while each Rust interpreter runs under `memcap 2G` with the same deadline. Independent stdout, stderr and status files under `final/` agree for all 20 programs and all 60 channels, including the unchanged registered `collection_copy` and `environment_symbol_after_store_changes` controls. `paired-agreement.json` and `verify-probes.py` retain the exact checks, while Q's differing channels remain available beside them.

The queue closure wording and original f report now point to this superseding HASITEM storage contract, while historical raw reports and inventory reason/kind/probe fields remain untouched. This wave adds no Deviation and does not broaden the existing licences, including the separately approved zero-count ruling. The original Q and other historical runtime pins are rehashed and preserved by the exclusive-create pin driver. No broad corpus/workspace gates, mutation experiment or performance profile was run here. Final performance acceptance and whole-task completion remain controller/user decisions after this handoff.
