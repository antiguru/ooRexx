**CLOSED: 3cd8cff5c.** Phase 9d Task 6.15 fix round 4: a file standard input is a persistent
stream with one read position, which CHARIN's start and LINEIN's line move; a start on a piped
standard input is 93.958 on both sides. Rows `console-charin-after-linein` and
`a_file_console_has_one_read_position` in `tests/input_oracle.rs`, oracle-identical.

# CHARIN after LINEIN on standard input returns too little

Found by Phase 9d Task 6.15 fix round 3 (report, Concern 1). After `linein()` on standard input,
`charin(, 3)` returns one character here (`w`) where the oracle returns one character (`l`): the
second argument is `start`, so the ask is for character 3, and rexxref makes `start` a position on
a persistent stream and an error on a transient one. It is the same on every binary back to the
perf pin, so it predates Task 6.15. Derive the rule from rexxref (one read position shared by
LINEIN and CHARIN, funct.xml CHARIN and LINEIN) before matching the oracle.
