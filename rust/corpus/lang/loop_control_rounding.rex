/* A controlled DO's control variable is rounded to NUMERIC DIGITS on every
   step, and the rounded value -- not the exact sum -- is what the next step
   adds to. The first loop is the witness: at DIGITS 3, 999 + 6 is 1005,
   which rounds to 1.01E+3, and the step after that adds 6 to 1010 and not to
   1005. An implementation that carries the exact sum forward instead prints
   1.01E+3 twice and trails one step behind from there on.

   FOR bounds every loop here: several of them stop advancing once rounding
   pins the control value, and would otherwise never reach their TO. */

numeric digits 3
do i = 999 by 6 for 6
    say i
end
say 'after' i

/* Rounding pinning the value: at DIGITS 3 the increment stops moving it. */
do j = 998 to 1005 for 6
    say j
end
say 'after' j

/* DIGITS narrowed by the body, so the width that was fine becomes too wide
   part way through. */
numeric digits 9
do k = 1 to 20 for 12
    if k = 3 then numeric digits 1
    say k
end
say 'after' k

/* FUZZ makes the TO comparison run at less than DIGITS precision, so the
   bound is reached earlier than an exact comparison would reach it. */
numeric digits 2
numeric fuzz 1
/* FOR 20 and not a rounder number: the count is itself read as a whole
   number at the DIGITS in force, and 200 is not one at DIGITS 2 (error
   26.3). */
do m = 1 to 10 for 20
    say m
end
say 'after' m
numeric fuzz 0

/* A non-integer BY, and a negative one. */
numeric digits 9
do n = 1 to 3 by 0.5
    say n
end
say 'after' n

do p = 10 to 1 by -3
    say p
end
say 'after' p
