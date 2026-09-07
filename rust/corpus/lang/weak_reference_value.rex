/* `WeakReference~value` -- `WeakReference::value` (`classes/WeakReferenceClass.cpp:217`),
   which is `resultOrNil(referentObject)` and nothing else.  What the collector
   does to a referent nothing else holds is not observable from Rexx, so the
   clearing half is witnessed under collect-on-every-allocation instead. */

o = .Object~new
w = .WeakReference~new(o)
say 'built' w~class~id w~string
say 'value' w~value~class~id (w~value == o) (w~value~identityHash == o~identityHash)
/* A second send to the answered object: it has to be the referent itself, not
   something merely of its class. */
say 'second-send' w~value~hasMethod('CLASS') w~value~isA(.Object)

/* `.nil` is a referent like any other -- `requiredArgument` accepts it and
   `resultOrNil` hands it straight back, so it is indistinguishable from a
   reference the collector has cleared. */
zn = .WeakReference~new(.nil)
say 'nil-referent' (zn~value == .nil) zn~value~class~id

/* `RexxObject::copy` carries the referent across. */
zc = w~copy
say 'copy' zc~class~id (zc~value == o)

/* A subclass keeps its own class, its own rendering and its own instance
   variables, and answers `value` all the same.  The arguments past the first
   go to `INIT`, not to the reference. */
zs = .W~new(o, 'tag')
say 'subclass' zs~class~id zs~string zs~label (zs~value == o)

/* A short argument list: the referent is required. */
signal on syntax name short
zz = .WeakReference~new
say 'unreached' zz

short:
say 'short' rc condition('C') condition('E')
exit 0

::class W subclass WeakReference
::attribute label
::method init
  expose label
  use arg label
