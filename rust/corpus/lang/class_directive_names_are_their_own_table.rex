/* Each kind of directive keeps its own table of names.

   The negative control for the duplicate refusals: R names a class and a
   routine in this file and neither collides, so a check keeping one table for
   every directive kind refuses it. B is here against a check that refuses a
   second ::CLASS whatever it is called.

   ::RESOURCE belongs in this row and cannot be: rexx-parse's
   every_corpus_program_tiles requires every byte of a corpus program to be
   covered by a clause node, and a ::RESOURCE body is source lines that no
   clause span covers -- measured, it fails byte by byte. Its rows are
   run/tests.rs's
   a_duplicate_resource_name_is_refused_and_a_distinct_one_is_not. */
say .R~m
say r()
say .B~m

::CLASS R
::METHOD m CLASS
  return 'the class R'

::ROUTINE R
  return 'the routine R'

::CLASS B
::METHOD m CLASS
  return 'a second class'
