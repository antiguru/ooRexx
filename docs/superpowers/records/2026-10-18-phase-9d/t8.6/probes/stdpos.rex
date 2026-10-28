q = 'position;position read;position write;position read line;position write line;position read char;position write char;position sys;seek;size'
s = .stream~new('STDIN'); x = s~chars
call show 'in0', s
x = s~linein
call show 'in1', s
x = s~charin
call show 'in2', s
call show '.in', .stdin
o = .stream~new('STDOUT'); x = o~open
call show 'out0', o
call show '.out', .stdout
call show '.err', .stderr
exit
show: procedure expose q
  use arg n, s
  out = n
  do w over q~makearray(';')
    out = out '['s~query(w)']'
  end
  call lineout 'STDERR', out
  return
