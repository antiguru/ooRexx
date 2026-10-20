/* A redirect target whose method parks while the clause writes to it: the
   output that arrives meanwhile and the child's end still reach the
   clause, and a target may issue a redirected command of its own. */
o = .Slow~new('sleep')
address system 'echo a' with output using (o)
say 'sleeping target' rc 'calls' o~calls
o = .Slow~new('long')
address system 'echo a; sleep 0.2; echo b; sleep 1.5; echo c' with output using (o)
say 'long write' rc 'calls' o~calls
o = .Slow~new('nested')
address system 'echo a; sleep 0.3; echo b' with output using (o)
say 'nested command' rc 'calls' o~calls
exit

::class Slow subclass OutputStream
::attribute calls
::attribute kind
::method init
  use arg self~kind
  self~calls = 0
::method lineout
  use arg line
  self~calls += 1
  select
    when self~kind == 'sleep' then call SysSleep 0.1
    when self~kind == 'long' & line == 'a' then call SysSleep 0.6
    when self~kind == 'nested' then do
      address system 'echo inner' line with output stem q.
      say 'inner' q.0 q.1
    end
    otherwise nop
  end
  say 'O:'line
  return 0
