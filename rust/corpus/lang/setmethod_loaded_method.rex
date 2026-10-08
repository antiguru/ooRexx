/* setMethod with a loadExternalMethod answer over a library procedure runs
 * it, blamed under the .nil scope. */
say .t~new~go
::class t
::method go
  self~setMethod('POS', .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos'))
  say 'set'
  return self~pos()
