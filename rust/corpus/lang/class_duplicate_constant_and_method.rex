/* A ::CONSTANT and a ::METHOD claiming one name on one side of one class.

   The constant occupies both dictionaries from a single directive, so it
   collides with a class-side ::METHOD and with an instance-side one alike;
   this file uses the class side. The report is the LATER directive's own
   kind -- ::METHOD's 99.902, not ::CONSTANT's 99.932 -- because the check
   takes its error code from the directive being parsed. */
say 'prolog'

::CLASS A
::CONSTANT c 5
::METHOD c CLASS
  return 'method'
