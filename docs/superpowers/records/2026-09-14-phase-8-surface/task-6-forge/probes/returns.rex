/* What a direct handler's answer becomes: RC, .RS, the RC( ) trace line.
   The traced commands are clauses of their own: `address env 'cmd'` under
   TRACE C echoes no clause line here (a recorded gap). */
call AddCmd 'xd', 'd'
address xd 'NULL'
say 'null' rc .rs (rc == .false)
address xd 'NUM'
say 'num' rc .rs
address xd 'ARRAY'
say 'array' rc~class~id .rs
address xd 'hello'
say 'echo' rc .rs
address xd
trace c
'NUM'
'NULL'
'ARRAY'
trace e
'NUM'
trace o
address
call AddCmd 'xx', 'x'
address xx 'hello'
say 'bad type' rc .rs
call AddCmd 'bash', 'd'
address bash 'replaced'
say 'bash' rc .rs
address xd
'implicit'
say 'implicit' rc
call AddCmd 'xd', 'r'
address xd 'FLAGS'
say 'replaced by redirecting' rc
::requires 'cmd' LIBRARY
