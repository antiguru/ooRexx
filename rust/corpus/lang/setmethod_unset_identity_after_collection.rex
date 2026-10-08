/* A method that unsets its own setMethod entry answers the same
 * .context~executable before and after a forced collection, under FLOAT,
 * under OBJECT, and on a class object. */
say .t~new~go
say .t~new~go2
say .t~go3
::class t
::method go
  self~setMethod('x', 'self~unsetMethod("X"); call gc "force"; e = .context~executable; h = e~identityHash; drop e; call gc "force"; f = .context~executable; return (f~identityHash = h) f~scope f~source[1]')
  return self~x
::method go2
  self~setMethod('x', 'self~unsetMethod("X"); call gc "force"; e = .context~executable; return e~scope e~source~items e~package~name~length > 0', 'OBJECT')
  return self~x
::method go3 class
  self~setMethod('x', 'self~unsetMethod("X"); call gc "force"; e = .context~executable; return e~scope e~source[1]')
  return self~x
