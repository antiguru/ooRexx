/* A package's imported routines take every ::REQUIRES ... LIBRARY first and
   then each ::REQUIRES, a name keeping its first entry: a library routine wins
   over a required package's public routine of the same name, whatever the
   source order, unless a package requiring the public routine first is itself
   required before the one carrying the library. */
say 'main' RxCalcSqrt(16) .context~package~findRoutine('RxCalcSqrt')~call(25)
say 'library first' libfirst()
say 'public first' pubfirst()
::requires 'pub.cls'
::requires 'rxmath' LIBRARY
::requires 'libfirst.cls'
::requires 'pubfirst.cls'
