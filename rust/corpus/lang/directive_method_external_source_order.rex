/* The entry point is resolved in the walk that answers every other
   translation-time refusal, in source order, so the ::METHOD above wins over
   the duplicate ::ROUTINE pair below it. */
say 'main ran'

::method m external 'LIBRARY REXX no_such_entry_point_xyz'

::routine dup
  return 1

::routine dup
  return 2
