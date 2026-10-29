**OPEN, queued 2026-10-29 by the controller from an external (codex) review of 9af69058d; Moritz 2026-10-29 "lgtm" to the placement: proposed after 9d (witness through the C API harness).**

# Native Buffer storage is not charged to the collector's byte trigger

`rexx-exec/src/dispatch/library/surface.rs:225` (`new_buffer`) boxes `NativeState::zeroed(length)` without charging its bytes, and survivor accounting omits it, while `new_mutable_buffer` charges its growth. Many large unreachable native buffers could exhaust memory before the object-count trigger runs. Witness: a native method allocating large buffers in a loop, peak RSS at two counts.
