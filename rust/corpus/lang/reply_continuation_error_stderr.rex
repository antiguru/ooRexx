/* The rest of a replied method fails: its traceback is on stderr and the
   exit status stays the sender's. */
d = .directory~new
say .k~new~m(d)
do until d~hasIndex('done')
  call SysSleep 0.01
end
say 'main ends'
exit 3
::class k
::method m
  use arg d
  reply 'replied'
  d['done'] = 1
  say 1/0
