/* FORWARD's legality check, which is asked before any option is evaluated and
   is 98.947 where GUARD's and REPLY's twins are 99.911 and 99.919. The
   ::ROUTINE shape rather than a program's own clause, because it carries the
   sending clause under the failing one and so pins the frame as well as the
   catalogue row. The `to` expression names a variable the routine never
   assigns, which would be a NOVALUE anywhere the check did not fire first. */
say 'a' r()
say 'never'

::ROUTINE r
  forward to (unassigned)
