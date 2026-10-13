say 'a'
call value 'TERM', 'xterm', 'ENVIRONMENT'
r = SysCls()
say 'b' r
call value 'TERM', 'dumb', 'ENVIRONMENT'
say 'c' SysCls()
call value 'TERM', '', 'ENVIRONMENT'
say 'd' SysCls()
call value 'TERM', 'nosuchterm', 'ENVIRONMENT'
say 'e' SysCls()
call value 'PATH', '/nonexistent', 'ENVIRONMENT'
say 'f' SysCls()
