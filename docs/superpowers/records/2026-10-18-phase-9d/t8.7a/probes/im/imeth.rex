.k~new~go
::class k
::method go
  m = .method~new('m', 'return 1')
  self~setMethod('M', m)
  say (self~instanceMethod('M') == m) self~instanceMethod('M')~class~id
  self~setMethod('N', 'return 2')
  say self~instanceMethod('N')~class~id
