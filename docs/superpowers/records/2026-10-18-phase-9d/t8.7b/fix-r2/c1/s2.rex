call on user ping name hu
call rp 'u'
say 'after user'
exit
rp: parse arg dd; raise user ping description (dd) return
hu:
  say 'hu' condition('d')
  call rp 'u2'
  signal off user ping
  say 'hu after signal off' condition('s')
  call rp 'u3'
  say 'hu end'
  return
