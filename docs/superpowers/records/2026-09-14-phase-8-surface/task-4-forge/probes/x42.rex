say 'main'
call stash
call deeper 40
exit
deeper: procedure
  arg n
  if n > 0 then return deeper(n - 1)
  say 'x' usestash()
  return 0
::requires 'forgea' LIBRARY
