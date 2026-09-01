/* A method setMethod attaches to one object is found before the class's own
   entry of the same name, and unsetMethod takes only the object's own away.
   usesem.rex cannot see either: it defines EXTRA nowhere else, so a search
   that asked the class first would print the same bytes. */
o = .k~new
say 'before' o~mm
o~shadow
say 'after' o~mm
say 'has' o~hasMethod('MM')
o~reveal
say 'revealed' o~mm
say 'has2' o~hasMethod('MM')
say 'other' .k~new~mm
b = .kid~new
b~shadow
say 'inherited' b~mm
b~reveal
say 'inherited-back' b~mm

::class k
::method mm
  return 'class-mm'
::method shadow
  self~setMethod('MM', 'return "object-mm"')
::method reveal
  self~unsetMethod('MM')

::class kid subclass k
