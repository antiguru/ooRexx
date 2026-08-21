/* provide.xml `xscope`: a scope is the methods and object variables defined
   for a single class, not including its superclasses, so the same method name
   is a different definition at each scope, and Method~scope answers which. */
say 'base' .base~method("M")~scope~id
say 'sub' .sub~method("M")~scope~id
say 'inherited-is-not-own' .sub~method("BASEONLY")

::class base
::method m
  return 'base'
::method baseonly
  return 'baseonly'
::class sub subclass base
::method m
  return 'sub'
