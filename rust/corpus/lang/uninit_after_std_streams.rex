/* A finalizer still reaches the standard streams after .local holds them.
   Touching .stdout mints the whole bundle, whose Stream objects must not be
   finalized ahead of the objects a program keeps: draining them first leaves
   every route a finalizer could write through already closed. */
zz = .stdout~string
keeper = .K~new
say 'built'

::class k
::method init
  .K~keep(self)
::method keep class
  expose bag
  use arg obj
  bag = obj
::method uninit
  say 'say reaches'
  zz = .stdout~lineout('stdout reaches')
  zz = .stderr~lineout('stderr reaches')
