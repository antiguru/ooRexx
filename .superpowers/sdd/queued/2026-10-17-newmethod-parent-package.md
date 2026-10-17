# Native `NewMethod` on source gives the method the caller's package

Found 2026-10-17 by the Phase 9c Task 9 re-review (`.superpowers/sdd/2026-10-15-phase-9c/task-9-rereview.md`).
The oracle's `LanguageParser::createMethod(name, buffer)` (`interpreter/parser/LanguageParser.cpp:134`)
builds the method from source with no context, so it has no parent package. For ordinary source
the crate's `NewMethod` falls back to the `.Method~new` send (`rexx-exec/src/dispatch/class_protocol.rs:361`),
which gives the method the caller's package as parent. Lines whose second line is the encoded tag
already take the direct path with no parent. The difference is read from the code. No native
witness measures it yet. The fix is to route all `NewMethod` input through the direct path, with
a witness that calls a routine only the caller's package defines (oracle: 43.1).
