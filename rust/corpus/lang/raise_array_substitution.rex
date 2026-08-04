/* RAISE SYNTAX ... ARRAY, the substitution list and its own trace lines
 * (4b Task 9), under trace i so the >A> lines are visible.
 *
 * THE OMITTED POSITION IS THE POINT. 40.4's message is "Too many arguments
 * in invocation of &1; maximum expected is &2.", and array ('R',,'X')
 * leaves &2 a hole: the oracle substitutes nothing there and never uses
 * 'X' at all. An implementation that closes the gap up instead prints
 * "maximum expected is X." -- readable, plausible, and wrong. That is a
 * stdout/stderr content difference, not a trace one, and it is why this
 * program's raise is left untrapped: the substituted message is only
 * observable in the report, condition('o') being 4c's.
 *
 * The trace lines it also pins, all measured: >A> fires TWICE per supplied
 * element (RaiseInstruction.cpp calls traceArgument on both sides of the
 * put into the array) and once, empty, for the omitted one; and >K>
 * "ARRAY" comes AFTER the elements, not before them.
 *
 * Determinism: no clock, no PID, no filesystem state. The error report
 * names this program's own path, which both interpreters are given. */
trace i
say 'before'
raise syntax 40.4 array ('R',,'X')
say 'unreachable'
