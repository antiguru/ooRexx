/* The ARRAY spelling of the same substitution list, which is why
 * raise_additional_array.rex has to exist beside it: the two build the same
 * list on the oracle and their reports are byte-identical, so a build that
 * implements one and not the other passes whichever one it has.
 *
 * The traces are not identical and that is the point of tracing both: an
 * element written in an ARRAY list gets two `>A>` lines and an omitted one
 * gets one, which is `RaiseInstruction.cpp:229`-`237` calling traceArgument on
 * both sides of the put, where a parenthesised list gets one `>A>` per written
 * element and a `>>>` of its own.
 *
 * Untrapped and traced for the same reasons the other program is, and in
 * corpus.rs's RAW_STDERR_COMPARISON for the same one. rc 216.
 */
trace i
raise syntax 40.4 description (7,8) array ('R',,'X')
