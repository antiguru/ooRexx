# The collection trigger does not see class weight

Found 2026-10-12 by the Phase 9a enhanced-semantics scout
(`.superpowers/sdd/2026-10-11-phase-9a/enhanced-semantics.md`, R17).

`.Thing~subclass('x')` in a loop: classes are collectable (Phase 5j, 819de57fe), but a crate class
weighs about 20 kB and no collection runs within 50000 creations; 200000 creations peak at
1399512 kB against the oracle's 116300 kB. Each collected class also leaves two emptied `Behaviour`
slots behind (`ClassGraph::expunge`, `rexx-classes/src/class_graph.rs:308`). The open item is
`docs/superpowers/plans/phase-5j-gate.md` section 6; the 6.1 byte trigger did not close it for
classes.

Owner: unassigned. Every `enhanced` call creates a class, so Task 8b inherits the cost.

Measured 2026-10-12 by Phase 9a Task 8b at `70d314bac`: a class `~subclass`, `~mixinclass` or `enhanced` builds is charged toward the next collection (128 bytes per name its behaviours hold); `sub50000.rex` peaks at 92224 kB, against 1045452 kB at `8d3aa11d9`. A `::CLASS` directive is not charged, and the emptied `Behaviour` slots remain.
