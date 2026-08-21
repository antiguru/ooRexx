/* INHERIT consumes every remaining token of its clause, so a keyword written
   after the mixin list is read as one more class name. PUBLIC resolves to
   nothing and the program is 98.909 naming it, not a syntax error and not a
   class declared PUBLIC. */
say 'main ran'

::CLASS M MIXINCLASS Object

::CLASS K INHERIT M PUBLIC
