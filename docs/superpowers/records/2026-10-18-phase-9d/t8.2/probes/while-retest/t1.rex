signal on syntax
do j = 1 while f(j)
  nop
end
exit
syntax:
  do line over condition('O')~traceback
    say '['line']'
  end
  do fr over condition('O')~stackframes
    say fr~line '['fr~traceline']'
  end
  exit
f: procedure
  use arg n
  say .context~stackframes[2]~traceline
  if n = 2 then return 1 / 0
  return 1
