address system 'echo a; sleep 1.13; echo b' with output using (.Bad~new)
say 'after' rc
exit
::class Bad subclass OutputStream
::method lineout
  use arg l
  say "O:"l
  if l == "a" then return 1 / 0
  return 0
