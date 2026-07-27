/* Whitespace is an operator in Rexx, and this is the case that catches
   parsers built on libraries that skip it by default.

   With a routine F in scope and a variable F also set:
     say f(1)   -- calls the routine
     say f (1)  -- variable F, abuttal-concatenated with the expression (1)

   Same tokens, one space, entirely different program. A combinator parser
   that treats whitespace as insignificant produces a working parser that is
   silently wrong here. See decision D10 in the rewrite plan. */

f = "VAR"
say f(1)
say f (1)
say f(1) f (1)

/* Abuttal with no space concatenates without a blank; a space concatenates
   with one.

   The third line is a trap worth keeping. `a''b` looks like the classic Rexx
   idiom for blank-free concatenation, but it prints "x", not "xy": `''b` is
   an EMPTY BINARY STRING LITERAL, because the b suffix binds to the
   preceding quote. So the line is a concatenated with "", and the variable b
   is never read at all. Same rule that makes say a"|"b fail outright -- here
   it does not fail, it silently computes something else. */
a = "x"
b = "y"
say a || b
say a b
say a"y"
say a''b

/* The same rule decides array-style references from concatenation. */
s.1 = "stem"
say s.1
say length("abc")
say length ("abc")

exit 0

::routine f
  use arg n
  return "ROUTINE-CALLED-WITH-" || n
