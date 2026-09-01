/* setMethod's default FLOAT scope is one variable pool per object, shared by
   all of that object's FLOAT methods and separate from the class's. One
   method cannot tell that from one pool per method, and one instance cannot
   tell it from the class's pool, so this asks with two methods and two
   instances. */
a = .k~new
a~mk
say 'one' a~w1
say 'two' a~r2
say 'class-pool' a~peek
b = .k~new
b~mk
say 'other' b~r2
say 'other-class-pool' b~peek

::class k
::method init
  expose fv
  fv = 'class-fv'
::method peek
  expose fv
  return fv
::method mk
  self~setMethod('W1', 'expose fv; fv = "written"; return "w:"fv')
  self~setMethod('R2', 'expose fv; return "r:"fv')
