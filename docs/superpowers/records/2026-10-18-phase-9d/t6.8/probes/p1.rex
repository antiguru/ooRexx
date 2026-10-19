-- novalue syntax on each list form
call t 'drop'
call t 'pe'
call t 'expose'
call t 'dropset'
return
t: procedure
  arg which
  signal on syntax name caught
  select
    when which = 'DROP' then drop (nolist)
    when which = 'PE' then call pe
    when which = 'EXPOSE' then .k~new~m
    when which = 'DROPSET' then do; okl = 'z'; z = 1; drop (okl); say 'dropset ok' symbol('z'); end
  end
  say which': not raised'
  return
caught:
  say which':' condition('c') condition('o')~code '['condition('o')~message']'
  return
pe: procedure expose (nolist2)
return
::class k
::method m
expose (nolist3)
::options novalue syntax
