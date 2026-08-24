/* Task 21: the REXX_DEFINED lock, one row per mutator so that the frame line
   names each of the five in turn. `.Array` is a class the image itself
   defines; a `::CLASS` of one's own is not, and the mutators succeed there
   (class_mutators_user_class.rex). The `::METHOD` below is what gives
   `.methods~z` something to answer -- without it the argument raises 97.1
   before DEFINE is ever sent. */
.Array~define("ZORK", .methods~z)

::method z
  say "hi"
