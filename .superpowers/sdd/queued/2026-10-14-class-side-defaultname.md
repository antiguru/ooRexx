**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** own probe: DN on the oracle, The WITHNAME class here.

# A class-side DEFAULTNAME is not consulted when a class renders

Source: Phase 9b1 Task 2 report (.superpowers/sdd/2026-10-14-phase-9b1/task-2-report.md), pre-existing.
A class whose metaclass overrides DEFAULTNAME renders as "The WITHNAME class" in the crate; the
oracle sends DEFAULTNAME and prints its answer ("DN"). Site: native_string in
dispatch/object_protocol.rs. Fix with a corpus witness checked against the oracle.
