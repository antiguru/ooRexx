a. = 'x'; v = 1
call r2 >a., >v
exit
r2: use arg >w., >u
  say 'r2' symbol('W.') symbol('U') w.1 u
  call r3
  return
r3: procedure expose w. u
  say 'r3' symbol('W.') symbol('U') w.1 u value('U') .context~variables~hasIndex('W.') .context~variables~hasIndex('U')
  return
