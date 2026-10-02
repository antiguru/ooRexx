# 2026-10-01-message-notify-single-slot

RESOLVED by Phase 6 S2-S5 Task 7 fix round 1: each Rexx activation keeps a single notify slot (`Activation::notify_message`); `re2a` and `re2b` answer as the oracle, and witness `rust/corpus/lang/message_notify_single_slot.rex` covers both forms.

Found by the Phase 6 S2-S5 Task 3 review (M2), ruled not that task's (queued 2026-10-01).

The oracle notifies only the message a Rexx activation last recorded in its single `notifyObject`
slot (`RexxActivation::setObjNotify` overwrites it; `MessageClass::dispatch`,
`classes/MessageClass.cpp:421`, sets it on the top frame, and native frames ignore it;
`execution/RexxActivation.cpp:2470` notifies that one object). This crate marks every message whose
synchronous send a SYNTAX failure ends (`dispatch/object_protocol.rs` `record_held`,
`Activity::failed_sends`), so a send made through another message's send marks both.

Probe `re2a` (fresh empty directory):

    m = .message~new(.t~new, 'boom')
    m2 = .message~new(m, 'send')
    signal on syntax name s
    m2~send
    s: say m~hasError m2~hasError m2~completed
    ::class t
    ::method boom
      return 1/0

Oracle: `1 0 0`, rc 0. This crate before the fix: `1 1 1`, rc 0.

Probe `re2b`:

    m3 = .message~new(.t~new, 'boom')
    m4 = .message~new(m3, 'send')
    m4~start
    m4~wait
    call SysSleep 0.3
    say m3~hasError m4~hasError (m3~errorCondition == m4~errorCondition) (m3~errorCondition == .nil) (m4~errorCondition == .nil)
    ::class t
    ::method boom
      return 1/0

Oracle: `0 1 0 1 0`. This crate before the fix: `1 1 1 0 0`. Stderr (the started report) identical, rc 0 both.
The hasError half predates Task 3 (base 9b06ca28a answers `1 1 1` on `re2a`); Task 3 added a
condition object consistent with it.
