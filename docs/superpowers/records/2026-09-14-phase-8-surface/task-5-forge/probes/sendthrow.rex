signal on syntax
say 'send:' SendThrow(.r~new)
exit
syntax:
  c = condition('o')
  say 'trapped' rc c~code Dtors()
  say 'position' c~position 'propagated' c~propagated
  exit
::requires 'reach' LIBRARY
::class r
::method run
  return Throw('0')
