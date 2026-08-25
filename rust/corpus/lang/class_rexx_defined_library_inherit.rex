/* Task 23 fix round 3: the REXX_DEFINED lock on a class the interpreter's
   own library declares rather than one Setup.cpp builds -- `.Alarm` is a
   `::CLASS` in `CoreClasses.orx`, and the oracle's image build flags it the
   same way, so this is 98.985 and not the 88.901 the same argumentless send
   to a user class gives. The `class_rexx_defined_*` rows beside this one send
   to `.Array`, a native class, and so cover one half of what carries the
   flag. */
.Alarm~inherit()
