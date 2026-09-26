/* Throw, then keep using the interpreter and the same library; unloader at termination */
do i = 1 to 3
  say 'try' i tryit() Dtors()
  say Again(i)
  say .m~new~go
  say 'stash' Stash(.thrower~new) Dtors()
end
say 'gc' gc('F')
say Again('end')
exit
tryit: procedure
  signal on syntax
  call Rethrow
  return 'no trap'
syntax: return 'trapped' condition('o')~code
::requires 'rr' LIBRARY
::class thrower
::method run
  signal on syntax
  call ThrowStashed
  return 'no trap'
syntax: return 'run trapped' condition('o')~code
::class benign
::method run
  return "ran"
::class m
::method magain external "LIBRARY rr MAgain"
::method go
  expose v
  r = self~magain(.benign~new)
  return 'm' v r
