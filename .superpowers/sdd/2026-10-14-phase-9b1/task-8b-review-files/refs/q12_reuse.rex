keep = .array~new
bad = 0
do i = 1 to 40
  r = mk(i)
  if i // 3 = 0 then keep~append(r)
  call filler 5
  if r~value \== copies('k', 10) || i then bad = bad + 1
end
call filler 300
do j = 1 to keep~items
  if keep[j]~value \== copies('k', 10) || (j * 3) then bad = bad + 1
end
say 'q12' keep~items bad
exit
mk: procedure
  use arg n
  y = copies('k', 10) || n
  t = >y
  u = >y
  return u
::routine filler
  use arg n
  do i = 1 to n; z = copies('x', 100000); end
  return
