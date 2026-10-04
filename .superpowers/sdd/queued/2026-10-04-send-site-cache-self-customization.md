# Send-site cache and related work from SELF's Customization paper

Design note, not scheduled: https://claude.ai/artifact/LyVy7vbvZBhCbd8d352cGH (Chambers and Ungar,
PLDI 1989, against the engine at 7790acc96). Measured with callgrind, libc subtracted: method
resolution costs 404 to 457 Ir per send (sendloop, dispatch, dispatchclass); the message name is
copied twice per send (174 Ir with malloc/free); an environment symbol costs 295 to 320 Ir per
reference; a value-returning method scans the named globals by string (193 Ir on dispatchclass).
rexxcps sends no messages and moves by none of this.

Ranked items: W0 complete the behaviour version stamp (`class_define` at `class_graph.rs:603-615`
mutates without a bump; prerequisite for any cache keyed on it); W1 monomorphic cache per
`Op::Send` site; W2 per-site environment-symbol cache; W3 interned message names; S2 dedicated root
for a method's return value; then W4 to W6 behind data. Each item carries files, invalidation, a
measurement plan and a kill criterion on the page.
