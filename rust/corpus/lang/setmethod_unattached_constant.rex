/* A .METHODS ::CONSTANT entry answers its value through setMethod. */
say .t~new~go
::constant fc 7
::class t
::method go
  self~setMethod('c', .methods['FC'])
  return self~c
