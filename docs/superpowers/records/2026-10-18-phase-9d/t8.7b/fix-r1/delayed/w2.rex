-- CALL OFF in the handler: USER ignored, HALT default action
call on user off name h
call rp 'one'
say 'after user'
call on halt name hh
call rh 'one'
say 'not reached'
exit
rp: parse arg dd; raise user off description (dd) return
rh: parse arg dd; raise halt description (dd) return
h:
  say 'h' condition('d')
  call rp 'two'
  call off user off
  say 'h after off'
  return
hh:
  say 'hh' condition('d')
  call rh 'two'
  say 'hh before off'
  call off halt
  say 'hh not reached'
  return
