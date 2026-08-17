/* One object, one name, two values at once. Both class methods are sent to
   .SUB, so the receiver is the same object each time; what differs is the
   scope each method was declared in, and that is what the pool is keyed on. A
   pool keyed on the object alone answers the second write to both reads. */
x = .sub~setsup
x = .sub~setsub
say .sub~readsup .sub~readsub
say .sup~readsup

::class sup

::method setsup class
  expose v
  v = 'set-by-sup'
  return 1

::method readsup class
  expose v
  return v

::class sub subclass sup

::method setsub class
  expose v
  v = 'set-by-sub'
  return 1

::method readsub class
  expose v
  return v
