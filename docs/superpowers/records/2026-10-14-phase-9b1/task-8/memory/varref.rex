parse arg n
do i = 1 to n
  call p
end
say 'done' y
exit
p: procedure
  y = 1
  call q >y
  return
q: use arg >z
  z = z + 1
  return
