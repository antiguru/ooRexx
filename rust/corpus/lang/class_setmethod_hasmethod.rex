/* A class object's own method answers hasMethod and is taken back by
 * unsetMethod. */
say .k~go
say .k~hasmethod('M2') .k~hasmethod('ZZ')
.k~kill
signal on syntax
say .k~m2
exit
syntax: say 'syntax' condition('o')~code
::class k
::method go class
  self~setMethod('m2', 'return 42')
  return self~m2
::method kill class
  self~unsetMethod('m2')
