/* setMethod from a class method gives the class object a method of its own. */
say .k~go
say .k~m2
::class k
::method go class
  self~setMethod('m2', 'return 42')
  return self~m2
