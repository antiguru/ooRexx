/* The copying family copies only at the class it is sent to: every subclass's
   existing behaviour is rebuilt in place, so an instance of a subclass sees a
   method defined on its parent after the instance was created.  ~inherit on
   the parent reaches it the same way, and nothing here is asked of the class
   the method was defined on. */
o = .Sub~new
say 'a' o~hasMethod('LATE')
.Base~define('LATE', .methods~late)
say 'b' o~hasMethod('LATE')
say 'c' .Sub~new~hasMethod('LATE')
say 'd' o~hasMethod('MXM')
.Base~inherit(.Mx)
say 'e' o~hasMethod('MXM')
say 'f' o~mxm

::method late
  return 'late-ran'

::class Base
::class Sub subclass Base

::class Mx mixinclass Object
::method mxm
  return 'mixin-ran'
