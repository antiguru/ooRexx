# Reference DESCRIPTION corpus registration diagnosis

The single mismatch is covered by approved Deviation 83, with no new runtime defect demonstrated. The retained release-corpus report gives identical first four NUMERIC/SIGNAL lines, then Rust prints the noisy DESCRIPTION conversion before USER delivery and the bad DESCRIPTION conversion before `s3 98.900 boom.`. The oracle never reads DESCRIPTION in this program, omits both conversions, and reaches the subsequent bad-reference ERROR call. Both sides have empty stderr and status 0, so the differing later reachability follows directly from the approved conversion failure at RAISE.

Deviation 83 at `docs/superpowers/plans/phase-4-exclusions.txt:3107` quotes Moritz's exact approval, “Follow your recommendation on both.”, and explicitly states that a raising conversion fails at RAISE. Its mapping is also explicit in `task-8.8f-dispatch.md:20`, while local rexxref `instrc.xml:2866-2874` requires the DESCRIPTION expression's character string value. Required-string forwarding through a reference is independently specified by `utilityclasses.xml:12631-12632` and existing Deviation 72, which applies the referenced value's required-string protocol everywhere. This does not license eager non-SYNTAX RC or ADDITIONAL rendering, and no such extension is proposed.

Current `run/condition/raise.rs:78-90` evaluates, roots and converts DESCRIPTION through `required_string_value`, then stores its bytes. `dispatch/reqstr.rs:450,516,525` follows the reference to its referent, and its fallback at line 202 sends STRING on that referent, reaching these classes' DEFAULTNAME methods and propagating their failures. The existing `tests/semantic_boundaries/description.rex` already contains the same reference USER-DESCRIPTION and ERROR-RC tail and pins noisy conversion, ERROR delivery, bad conversion and a caught SYNTAX. The oracle's `RaiseInstruction.cpp:206-209` instead evaluates and traces DESCRIPTION without a required-string conversion, explaining its omitted sends when tracing is disabled.

## Smallest complete repair proposal

Preserve all executable lines of `corpus/lang/reference_conversion_raises_in_clause.rex` and correct only its two-line comment to state that DESCRIPTION converts at RAISE under Deviation 83. Move its phase-9 registration to `corpus/unfiled.txt`, following the existing complete `raise_untraced_renders_nothing.rex` registration. Add the complete source and retained stdout, empty stderr and explicit status 0 as a new semantic-boundary fixture, with a source-equality assertion against the corpus file. The existing runner then exercises it normally and with actual collection at every allocation, without adding or renaming a Rust test function.

A DESCRIPTION-free differential companion must preserve the original NUMERIC/SIGNAL cases and independently execute both noisy/bad reference ERROR cases that the full ruled witness can no longer reach. The scratch candidate additionally exercises both references through USER ADDITIONAL and a bad reference as SYNTAX RC, keeping these distinct from the approved DESCRIPTION timing. It reuses the existing classes and handler shapes, and is registered in phase-9 in place of the ruled program. This candidate has not been executed or assigned oracle expectations; after the freeze ends, its separate stdout/stderr/status must agree with the oracle before registration is accepted.

The test-only file set is as follows, with no runtime edits or new Deviation entry required:

* `rust/corpus/lang/reference_conversion_raises_in_clause.rex`: comment only, all executable lines preserved.
* `rust/corpus/lang/reference_conversion_without_description.rex`: new differential companion.
* `rust/corpus/phase-9.txt`: replace the registration and adjust its comment.
* `rust/corpus/unfiled.txt`: add the original path with Deviation 83 and its complete semantic fixture owner.
* `rust/crates/rexx-exec/tests/semantic_boundaries.rs`: add the full-source equality assertion.
* `rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.{rex,stdout,stderr,status}`: four complete ruled fixture files.
* `rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_raises_in_clause.txt`: regenerate through the established oracle driver for the corrected comment.
* `rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_without_description.txt`: oracle-generate for the new companion.

The existing SOURCELINE test scans every lang program, including unfiled ones, and `corpus::every_lang_program_is_run_or_named_unfiled` requires disjoint, complete registration. Expected population changes are 1242 differential programs unchanged, four unfiled programs instead of three, and 42 semantic fixtures instead of 41, with the same Rust test names. Focused later proof should cover the companion's oracle channels, the full semantic normal/stress target, SOURCELINE and corpus registration before controller-owned final gate resumption. No build, probe, test, Git mutation or tracked edit occurred during this diagnosis.

## Retained evidence and scratch

The exact observed channels come from `/tmp/codex-p9d-t88-final-gates/release-corpus.stderr`, with its SHA256 recorded in `provenance.json` beside this document. The scratch corrected full source preserves executable lines byte-for-byte, and its three channel files are extracted directly from that retained report, not inferred or rerun. `reference_conversion_without_description.rex` is an unexecuted candidate for later verification. All proposal files remain exclusively under `/tmp/codex-p9d-t88-reference-description-diagnosis/`; HEAD remains the supplied frozen `8b7767ed3fda908fffe75181b2c8bd12dd7509f8`.
