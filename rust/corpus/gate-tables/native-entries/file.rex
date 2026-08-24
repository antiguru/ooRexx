/* Sending to a LIBRARY REXX entry point the phase that owns the file system
   has not built. The bind succeeds, so 'main' prints and the refusal is the
   send's -- the entry points this phase does implement sit in this same
   family, so a program here also separates them from it. */
say 'main'
say .k~probe

::class k

::method probe class external 'LIBRARY REXX file_exists'
