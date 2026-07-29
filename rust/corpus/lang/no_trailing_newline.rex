/* no_trailing_newline.rex -- the SOURCELINE gate criterion names a file
   whose last line has no terminator. This is that file: keep the missing
   newline on the last line, it is the point. Created for the phase 3 gate,
   because every other corpus program ends with one (measured 2026-07-29). */
quiet = 1
if quiet then nop
else say "unreached"