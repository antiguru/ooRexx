/* A package built with this one as its context finds this package's own
   routines before any routine merged into itself, through a call and through
   findRoutine: ahead of a ::REQUIRES ... LIBRARY routine and of a required
   package's public routine of the same name. */
r = .Routine~newFile('r.rex', .context~package)
r~call
::routine rxcalcsqrt
  return 'main' arg(1)
::routine pubname
  return 'main' arg(1)
::routine mainr
  return 'mainr'
