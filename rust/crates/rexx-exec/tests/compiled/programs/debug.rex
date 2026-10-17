trace ?r
say 'a'
say 'b'
call x
interpret 'say "c"; say "d"'
say trace()
exit
x: say 'x'; return
