# Dispatch seam consumer-list diagnosis

The retained target failed because its explicit consumer-file list did not follow three approved responsibility moves. Frozen HEAD is `6b6f3fb460049c6ef63f565ceb78e11b5582f165`, and `/tmp/codex-p9d-t88-final-gates-resume/debug-118-dispatch_seam.json` records status 101, five passing tests and one failing test in 0.164 seconds. Every one of the nine rejected occurrences is an existing `_cleared: Cleared` function parameter moved from an already permitted source file. No rejected occurrence constructs, duplicates or newly routes a clearance, and no production correction is indicated by this failure.

## Provenance of all nine occurrences

The table's old coordinates refer to the immediate parent of the named move commit. Current coordinates are the exact nine sites in the retained failure. Read-only extraction of each function at the move parent and move commit compared equal after removing only indentation and the changed `pub(...)` visibility spelling. Current blame attributes all nine parameter lines to those move commits, even where later functional changes altered the body or shifted its location.

| Current site under `src/` | Function | Move commit | Previous permitted site under `src/` |
| --- | --- | --- | --- |
| `dispatch/hash/removal.rs:168` | `native_hash_empty` | `d7091492ceacea134a2b576fd2e11331d1c41cd6` | `dispatch/hash.rs:1244` |
| `dispatch/hash/removal.rs:195` | `native_hash_remove_item` | `0f48350f86f324e88809ed64f37c4dc84b3ee9c3` | `dispatch/hash.rs:1499` |
| `dispatch/method_entry.rs:25` | `Interp::begin_method` | `3f67e05145f4d5f41b1255175f46ac102913a362` | `dispatch.rs:3521` |
| `dispatch/stream/query.rs:22` | `query_exists` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:309` |
| `dispatch/stream/query.rs:90` | `query_size` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:366` |
| `dispatch/stream/query.rs:145` | `query_time` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:410` |
| `dispatch/stream/query.rs:179` | `query_streamtype` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:444` |
| `dispatch/stream/query.rs:208` | `query_handle` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:473` |
| `dispatch/stream/query.rs:236` | `query_position` | `f7d6e8d5ef2069a7517ff3f4967cab00f68744d0` | `dispatch/stream.rs:2295` |

The method-entry move is the earlier E2 extraction, while removal and stream-query moves belong to f and its whole-review fix wave. Hash imports its two moved native functions at `dispatch/hash.rs:19-21` and registers them in the existing Table/IdentityTable method rows at lines 2090, 2099, 2127 and 2151. Stream reexports all six query entry points at `dispatch/stream.rs:19-21`, and `dispatch/native.rs:203,245,251,257,281,287` still binds their original public surface. `dispatch.rs:2729-2754` obtains the clearance from the single seam call and passes it into `begin_method`; the native dispatch arms below pass the same owned token into native callbacks.

## Intended boundary and minimal repair

D45's producer boundary remains `dispatch.rs::seam`, whose private token field prevents construction outside that module. The test's consumer boundary is deliberately an explicit file enumeration, as the comment at `tests/dispatch_seam.rs:198` says, rather than permission for every descendant of dispatch. The retained run passes `dispatch_passes_through_exactly_one_chokepoint`, `the_seam_module_holds_one_struct_one_enum_and_one_function` and `the_protected_question_is_asked_only_inside_the_seam`, preserving its independent producer and security checks. Prior move validation ran refusal/representation/owner/coverage targets, as the final-fix report records, but did not include this separate `dispatch_seam` target.

The smallest justified repair changes only `rust/crates/rexx-exec/tests/dispatch_seam.rs` by adding these three concrete destinations to `CLEARANCE_CONSUMERS`, increasing its explicit entries from 25 to 28. Keep the existing parent entries because they still name the token, retain every existing comment and assertion, and make no production, visibility, seam-construction or call-path change. Do not replace the list with a `src/dispatch/` prefix, recursive-module exemption or all-source allowance. The previous `669b701cb` repair added `semaphore.rs` explicitly for the same parameter-only consumer rule.

```rust
    "src/dispatch/hash/removal.rs",
    "src/dispatch/method_entry.rs",
    "src/dispatch/stream/query.rs",
```

A necessary negative control should exercise the actual allow-list predicate, because the existing present/absent-token test validates source scanning but does not reject a broad path exemption. Within `the_seam_token_is_named_only_by_the_dispatch_module`, bind the existing predicate once and use it both for synthetic rejected paths and the actual site filter. The following scoped replacement preserves the existing matching semantics and all six test names, while rejecting an unlisted file outside dispatch and unlisted siblings below each newly allowed module area. A future broad-prefix shortcut would then fail the same assertion without injecting a token into production source.

```rust
    let is_allowed = |site: &str| {
        CLEARANCE_CONSUMERS.iter().any(|file| site.contains(file))
    };
    for site in [
        "/synthetic/src/unlisted.rs:1",
        "/synthetic/src/dispatch/unlisted.rs:1",
        "/synthetic/src/dispatch/hash/unlisted.rs:1",
        "/synthetic/src/dispatch/stream/unlisted.rs:1",
    ] {
        assert!(
            !is_allowed(site),
            "an unlisted clearance consumer was allowed: {site}"
        );
    }
    let elsewhere: Vec<&String> = sites
        .iter()
        .filter(|site| !is_allowed(site))
        .collect();
```

## Verification boundary

The retained failed command and all original channels remain untouched, and this diagnosis ran no test, build, probe or source mutation. Read-only Git history, blame, source inspection and pure-move function comparisons support the provenance above. Once the controller releases the freeze and authorizes the policy-test repair, the focused proof is the complete six-test `dispatch_seam` target plus formatting and affected-target lint, retaining the negative path assertions. Source/index checks currently remain quiet at the frozen HEAD, and only this scratch report was written.
