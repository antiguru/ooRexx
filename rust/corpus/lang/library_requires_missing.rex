/* ::REQUIRES ... LIBRARY names a shared object, not a package file, and a
   name nothing loads is refused before the program's own first clause: the
   SAY below never prints. */
say 'the prolog ran'
::requires 'zorkolib' LIBRARY
