/* A call looks its routine up once its arguments have run, and keeps what
   findRoutine found then: an argument calling an external file merges that
   file's public routine of the call's own name into the package, in front of
   the routine a ::REQUIRES ... LIBRARY merged into the package's context. The
   same through a function call, a CALL and an assignment. */
r = .Routine~newFile('r.rex', .context~package)
r~call
::requires 'rxmath' LIBRARY
