/* join with failing DEFAULTNAME, no trap */
say .array~of(1, .bad~new)~makestring
say 'not reached?'
::class bad
::method defaultname
  return 1 + 'a'
