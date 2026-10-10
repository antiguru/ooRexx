# `'12'~translate('', '')` blanks the string

Found by Phase 6.1 scout D (2026-10-07, `tr_base`). Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both. The built-in function form agrees (`b8_translate_bif.rex`: `translate('12','','')` is `[12]` on both).

Probe `b8_translate_empty.rex`, run from a fresh empty directory:

    say '['||'12'~translate('','')||']'

Oracle, rc 0:

    [stdout]
    [12]
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    [  ]
    [stderr]
    (empty)

The MutableBuffer method agrees, and the oracle blanks there: `mb = .mutablebuffer~new('12'); say '['mb~translate('','')~string']'` prints `[  ]` on both (`b8_translate_mb.rex`), so a fix belongs to the String method alone.

Suspected site: `translate_in_table` (`dispatch/buffer.rs`), which the String method `native_string_translate` (`dispatch/string.rs`) calls and which reads an empty input table as no input table; the BIF path (`builtin/string.rs` `translate`) keeps the empty table.
