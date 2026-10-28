m = .method~new('MM', 'trace a; nop')
say m~package~name
.k~new~run2(m)
r = .routine~new('RR', 'trace a; nop')
say r~package~name
r~call
.k~define('DD', 'trace a; nop')
.k~new~dd
::class k
::method run2
  use arg m
  self~run(m)
