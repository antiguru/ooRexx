/* What a ::CONSTANT accessor answers, one row per form the directive has.

   A value omitted takes the constant's own name, which is the token's value:
   already upcased for a symbol and left alone for a literal, so the two name
   forms are a pair rather than one row. The signed form drops the blank. An
   expression's value is whatever it evaluated to, and it is a String here
   rather than the number the expression built.

   The accessor is installed on the class side and the instance side alike
   from one directive, and ~hasMethod is what the class side can be asked
   about. Its argument bound is the last row: a constant takes none. */
say .A~c1
say .A~c2
say .A~c3
say .A~"c4"
say .A~c5
say .A~c6
say .A~c6~class~id
say .A~hasMethod("C1")
signal on syntax name toomany
say .A~c1(1)
say "the argument was accepted"
exit
toomany:
say "arg refusal" condition('E')

::CLASS A
::CONSTANT c1 5
::CONSTANT c2 'some text'
::CONSTANT c3
::CONSTANT "c4"
::CONSTANT c5 - 5
::CONSTANT c6 (2+3)
