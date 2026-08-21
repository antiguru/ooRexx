/* A metaclass donates its own instance methods to the class side of the
   classes built from it, so a build that accepts METACLASS and ignores it
   answers 97.1 instead of this line. */
say .k~classSideHi
::class meta subclass class
::method classSideHi
  return 'from the metaclass'
::class k metaclass meta
