/* Canonicalisation on arithmetic. The subtle rule is that trailing zeros
   AFTER a decimal point are significant and preserved -- 1.50 + 0 is 1.50,
   not 1.5 -- while leading zeros, a bare trailing point, a unary plus and
   surrounding whitespace are all stripped. Zero is the exception that
   collapses completely: -0 and 0.0 both become 0. */
list = .array~of("1", "1.0", "01", "1.", ".5", "+5", "-0", "0.0", "0.00",,
                 "1e5", "1E+5", "1e-5", " 7 ", "1.50", "000.500", "1e0",,
                 "-1.50", "00.00", "+0.0", "12.3400")
do v over list
  say "[" || v || "] -> [" || (v + 0) || "]"
end

/* The preserved zeros survive arithmetic and affect the result's form. */
say 1.50 + 0.50
say 1.5 + 0.5
say 1.50 * 2
say 2.0 - 2.0
