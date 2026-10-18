.k~new~go
::class k
::method go
  self~setMethod('m', 'return "m ran"')
  say self~m
  say self~run(.method~new('x', 'return "run ran"'))
