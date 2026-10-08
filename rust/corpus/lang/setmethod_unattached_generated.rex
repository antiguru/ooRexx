/* .METHODS entries a directive generates: an attribute pair, a DELEGATE and an
 * ABSTRACT, each run through setMethod. */
say .t~new~go
::method fa attribute
::method fd delegate zz
::method fab abstract
::class t
::method go
  expose zz
  zz = 'abcd'
  self~setMethod('a', .methods['FA'])
  self~setMethod('a=', .methods['FA='])
  self~a = 5
  r = self~a
  self~setMethod('length', .methods['FD'])
  r = r self~length
  signal on syntax
  self~setMethod('ab', .methods['FAB'])
  r = r self~ab
  return r
syntax: return r 'syntax' condition('o')~code
