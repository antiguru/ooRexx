/* Task 16's mutation-testing witness: a number's rendering is fixed by the
   NUMERIC FORM in force when it is CREATED, never by the FORM in force when
   it is first DISPLAYED. See mutation_digits_at_render.rex's own comment for
   why the Controlled loop's control variable is the one construct in 4a
   where this window is observable at all: every other write site renders
   eagerly, before a later NUMERIC FORM instruction can run.

   DIGITS 3 forces exponential notation for 12000 (five significant digits),
   and an exponent of 4 -- not a multiple of 3 -- is chosen deliberately, so
   ENGINEERING and SCIENTIFIC notation visibly disagree: engineering backs
   the exponent off to the nearest lower multiple of 3, widening the
   mantissa, while scientific always keeps exactly one digit before the
   point.

   Measured: the oracle prints "12.0E+3" (engineering, the FORM in force
   when `i` is bound for its one pass, DO ... FOR 1), not "1.20E+4"
   (scientific, the FORM in force at the `say`). Mutating the render site to
   use the current setting instead of the value's own created_form flips
   this to "1.20E+4". */
numeric digits 3
numeric form engineering
do i = 12000 for 1
  numeric form scientific
  say i
end
