/* When does a number display in exponential form? Measured, not guessed --
   and the first measurement was wrong, so this program covers the case that
   exposed it.

     positive:  E-notation once the ADJUSTED exponent (that of the most
                significant digit) is >= DIGITS
     negative:  E-notation once the RAW exponent (that of the least
                significant digit) is <= -(2 * DIGITS + 1)

   The two sides use different exponents. They coincide only when the mantissa
   is a single digit, which is why probing with 1eN values alone suggests both
   sides use the adjusted one. They do not: 1e-18 and 10e-19 are the same
   value and print differently. */

do d = 1 to 9 by 2
  numeric digits d
  say "digits" d
  say "  + boundary:" ("1e" || (d - 1)) + 0 "|" ("1e" || d) + 0
  say "  - boundary:" ("1e-" || (2 * d)) + 0 "|" ("1e-" || (2 * d + 1)) + 0
end

/* Same value, different spellings, different display form. */
numeric digits 9
say "1e-18   ->" 1e-18 + 0
say "10e-19  ->" 10e-19 + 0
say "1.0e-18 ->" 1.0e-18 + 0
say "123e-19 ->" 123e-19 + 0
say "1000e-19->" 1000e-19 + 0
say "12345e-20 ->" 12345e-20 + 0
