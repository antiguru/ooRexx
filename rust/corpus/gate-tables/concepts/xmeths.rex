/* provide.xml `xmeths`: the search order for a message is the object's own
   class, then a superclass of it, then UNKNOWN, and a NOMETHOD condition when
   there is no UNKNOWN. (The first documented step, a method the object itself
   defines with setMethod or enhanced, is per-object and has its own section.) */
say 'own-class' .sub~m
say 'superclass' .sub~basem
say 'unknown' .sub~zork
say 'nomethod' .plain~zork

::class base
::method m class
  return 'base m'
::method basem class
  return 'base basem'
::class sub subclass base
::method m class
  return 'sub m'
::method unknown class
  use arg name
  return 'unknown' name
::class plain
