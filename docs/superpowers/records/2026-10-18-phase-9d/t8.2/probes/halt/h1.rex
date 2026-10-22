address system 'echo a; sleep 1.13; echo b' with output using (.Bad~new)
say 'after' rc
exit
::class Bad subclass OutputStream
::method lineout
  return 1 / 0
