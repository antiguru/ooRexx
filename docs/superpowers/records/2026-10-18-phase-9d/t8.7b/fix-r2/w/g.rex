call on user ping name h
n = 0
call rp 'first'
say 'after, n =' n
exit
rp: parse arg dd; raise user ping description (dd) additional (dd'-add') return
h:
  n = n + 1
  say 'h' n arg(1)~description arg(1)~additional
  if n = 1 then do
    call rp 'second'
    call rp 'third'
    call rp 'fourth'
  end
  return
