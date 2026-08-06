/* `>.>`, the PARSE template placeholder's own line, and the one fact about
   PARSE's trace that only two modes together can pin.

   A target's own value line is a CHOICE of prefix, not two independent gates.
   `ParseTrigger::parse` assigns -- whose own `traceAssignment` is the
   intermediates-level `>=>` -- and then emits `traceResult` only if
   intermediates are NOT being traced. So the same clause traces `>=> P <=
   "one"` under TRACE I and `>>> "one"` under TRACE R, never both, and an
   engine with two independent gates emits both under I. Running the identical
   template under both modes is what makes that visible; either mode alone
   passes against the wrong implementation.

   `>.>` is intermediates-level, so it appears in the TRACE I section and not
   in the TRACE R one -- which is the second thing the two sections pin
   together, and the reason a TRACE R probe once recorded the prefix as
   unreachable.

   The second PARSE in each section has a placeholder that consumes NOTHING:
   its `>.>` line is still emitted, as `>.>   ""`, rather than being skipped.

   No `parse source` anywhere: its `>K>` line carries the program's own
   absolute path, which cannot go in a committed expectation. */

trace i
parse value 'one two three' with p . q
parse value 'four' with r . s
parse value 'abcdefghij' with t 5 u
trace off
trace r
parse value 'one two three' with p . q
parse value 'four' with r . s
parse value 'abcdefghij' with t 5 u
trace off
say p||q||r||s||t||u
