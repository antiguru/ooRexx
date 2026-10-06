o = .g~new; o~start("setlater"); say "ok" o~waitfor
::class g
::attribute flag
::method init; expose flag; flag = 0
::method setlater; expose flag; call syssleep 0.1; flag = 1
::method waitfor; expose flag; guard on when flag = 1; return "seen"
