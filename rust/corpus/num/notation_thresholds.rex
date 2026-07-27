/* When does a number display in exponential form? Measured, not guessed.

     positive:  E-notation once the exponent >= DIGITS
     negative:  E-notation once the exponent <= -(2 * DIGITS + 1)

   The asymmetry is the point -- at DIGITS 9 a number stays in plain form down
   to 1e-18 but switches at 1e+9 -- and an implementation that picks one
   threshold for both directions is silently wrong across most numeric output.
   Verified at DIGITS 1, 3, 5 and 9. */

do d = 1 to 9 by 2
  numeric digits d
  say "digits" d
  say "  + boundary:" ("1e" || (d - 1)) + 0 "|" ("1e" || d) + 0
  say "  - boundary:" ("1e-" || (2 * d)) + 0 "|" ("1e-" || (2 * d + 1)) + 0
end
