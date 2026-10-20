o = .G~new
m = o~start('hold')
call SysSleep 0.1
call time 'r'
address system 'echo a; sleep 0.2; echo b; sleep 0.2; echo c' with output using (o)
say 'done' rc 'elapsed>0.9' (time('e') > 0.9)
m~result
exit
::class G subclass OutputStream
::method hold
  call SysSleep 1
  say 'released'
::method lineout
  use arg l
  say 'O:'l
  return 0
