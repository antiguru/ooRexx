/* DEFAULTNAME raising SYNTAX, rendered by an array join */
signal on syntax
say .array~of(.bad~new)~makestring
say 'not reached?'
exit
syntax: say 'trapped' condition('o')~code condition('o')~message; exit
::class bad
::method defaultname
  return 1 + 'a'
