call stash
.local~keep = .thing~new
say 'main'
::requires 'forgea' LIBRARY
::class thing
::method uninit
  say 'uninit local same' same()
  say 'use' usestash()
