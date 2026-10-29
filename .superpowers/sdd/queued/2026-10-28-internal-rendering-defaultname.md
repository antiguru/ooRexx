**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7c).**

# Internal renderings do not send DEFAULTNAME

Found by Phase 9d Task 8.7a (task-8.7a-report.md, Concerns 1; confirmed by task-8.7a-review.md); the instance half predates the task.
A trace value line, an error message's object insert and an Array's `makeString`/`toString` join print the default name here, where the oracle's `stringValue` sends `DEFAULTNAME` to a non-primitive object: `>>>   "DN"`, `Object "DN" does not understand`, `DN, The PLAIN class` there. Those renderings go through `string_value_text`, which does not send.
Probes: docs/superpowers/records/2026-10-18-phase-9d/t8.7a/probes/dn/dn4.rex (a class), dn5.rex (an instance). Derive from rexxref (DEFAULTNAME, STRING, trace output) before following the oracle.
