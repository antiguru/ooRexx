/* The mixin's own method has to reach k, so a build that accepts INHERIT
   and adds no superclass edge answers 97.1 rather than this line. */
say .k~m
::class mx mixinclass object
::method m class
  return 'from the mixin'
::class k inherit mx
