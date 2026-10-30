say 'before'
signal on user sample name handler
raise user sample description (.key~new)
say 'unreached'
handler:
say 'handler'
say 'first' condition('d')
say 'second' condition('d')
exit
::class key
::method makeString
  say 'make'
  return 'description'
