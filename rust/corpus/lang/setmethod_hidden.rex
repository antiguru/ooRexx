/* setMethod with no method argument stores the oracle's .nil under the name
   and hides whatever the class answers, which is not what unsetMethod does:
   hasMethod reports 0 and the send is 97.1 even though the class defines the
   method. */
o = .k~new
say 'before' o~mm
say 'has' o~hasMethod('MM')
o~hide
say 'has2' o~hasMethod('MM')
say 'other' .k~new~mm
o~reveal
say 'revealed' o~mm

::class k
::method mm
  return 'class-mm'
::method hide
  self~setMethod('MM')
::method reveal
  self~unsetMethod('MM')
