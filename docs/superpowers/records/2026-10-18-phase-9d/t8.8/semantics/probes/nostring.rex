call check 'say s.[.key~new]'
call check 's.[.key~new] = "stored"'
call check 'say s.~hasIndex(.key~new)'
call check 'parse value "abc" with (.key~new) result'
exit
check:
  signal on nostring name caught
  interpret arg(1)
  say 'unreached'
  return
caught: say 'NOSTRING'; return
::class key
