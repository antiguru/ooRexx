/* A bound ::METHOD EXTERNAL is not a gap, so it no longer preempts an
   install-time failure standing above it: the ::CONSTANT's own divide is what
   answers. The same file with a ::ROUTINE EXTERNAL in that position is
   refused here instead, which is the cost staged_gap's doc records. */
say 'main ran'

::class a

::constant kk (1/0)

::method m external 'LIBRARY REXX file_separator'
