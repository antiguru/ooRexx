/* provide.xml `chsrod`: a colon and a class symbol after a message name change
   where the method search starts, so a subclass's method can call the
   superclass's method of the same name. */
say .savings~type

::class account
::method type class
  return "an account"

::class savings subclass account
::method type class
  return self~type:super "(savings)"
