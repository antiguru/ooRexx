/* run with a method that is neither a string, an array nor a Method object:
 * 93.974. */
say .t~new~go
::class t
::method go
signal on syntax
return self~run(.object~new)
syntax: return rc condition('O')~code
