/* HALT interrupts a loop that traps a stem-default cycle's 11.1 */
signal on halt
s. = >s.
n = 0
again:
  signal on syntax
  say s.
  say 'not reached'
syntax:
  n = n + 1
  signal again
halt: say 'HALT' condition('C') 'after' (n > 0); exit
