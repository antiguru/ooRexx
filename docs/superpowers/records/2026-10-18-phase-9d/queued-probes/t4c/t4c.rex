o = .k~new
o~go
::class k
::method go
  self~setMethod('m', 'trace a; nop')
  self~m
