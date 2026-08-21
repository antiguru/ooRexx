/* Table C method rows: Validate, class arm -- .Validate~hasMethod("M") for
   every method corpus/docs/class-methods.txt documents on this arm,
   one line per row and in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'class' .Validate~hasMethod("classType")
say 'class' .Validate~hasMethod("length")
say 'class' .Validate~hasMethod("logical")
say 'class' .Validate~hasMethod("nonNegativeNumber")
say 'class' .Validate~hasMethod("nonNegativeWholeNumber")
say 'class' .Validate~hasMethod("number")
say 'class' .Validate~hasMethod("numberRange")
say 'class' .Validate~hasMethod("position")
say 'class' .Validate~hasMethod("positiveNumber")
say 'class' .Validate~hasMethod("positiveWholeNumber")
say 'class' .Validate~hasMethod("requestClassType")
say 'class' .Validate~hasMethod("wholeNumber")
say 'class' .Validate~hasMethod("wholeNumberRange")
