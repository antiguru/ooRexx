**CLOSED: 0a055271c.** Closed 2026-10-14 by Phase 9b1 Task 2; the probe agrees on both engines.

# A class-side ::method string is not sent for a class object's string value

Source: Phase 9b Task 3 review (.superpowers/sdd/2026-10-13-phase-9b/task-3-review.md), pre-existing.
The oracle sends a class-side `::method string` when a class object needs its string value. The crate
prints "The K class". The likely site is the reqstr.rs Instance gate. Fix: route class objects through
the sent STRING when the metaclass overrides it, with a corpus witness.
