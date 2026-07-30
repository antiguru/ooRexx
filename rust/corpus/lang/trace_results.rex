/* TRACE R (RESULTS) output. Like TRACE I, this goes to stderr while SAY
   output stays on stdout -- the two streams must be captured and compared
   separately. */
trace r
x = 1 + 1
y = x * 3
if y > 5 then say "big"
trace off
say "done" y
