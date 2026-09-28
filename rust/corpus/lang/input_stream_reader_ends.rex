-- An InputStream reader's LINEIN ending the input without a RETURN tail:
-- a tail-less RAISE of NOTREADY, USER or HALT, and a bare RETURN, each end
-- it, as does a RAISE ... EXIT; the issuing activation's traps never run.
call on notready name nr
call on user stop name us
call on halt name hh
do kind over .array~of('NOTREADY', 'USER', 'HALT', 'BARE', 'EXIT', 'DEEP', 'VALUE')
  out = .array~new
  address system 'cat' with input using (.Src~new(kind)) output using (out)
  say kind rc out~items out~makestring('l', '|')
end
exit
nr: say 'notready handler ran'; return
us: say 'user handler ran'; return
hh: say 'halt handler ran'; return
::class Src inherit InputStream
::method init
  expose kind lines
  use strict arg kind
  lines = .array~of('a', '', 'c')
::method linein
  expose kind lines
  if lines~items > 0 then do
    v = lines[1]
    lines~delete(1)
    return v
  end
  select
    when kind = 'NOTREADY' then raise notready
    when kind = 'USER' then raise user stop
    when kind = 'HALT' then raise halt
    when kind = 'BARE' then return
    when kind = 'EXIT' then raise user stop exit 'ignored'
    when kind = 'DEEP' then call ending
    otherwise raise user stop return 'last'
  end
  return 'never'
ending:
  raise notready description 'from an internal call'
