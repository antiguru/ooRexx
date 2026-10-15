parse arg n
do i = 1 to n
  call p
end
say 'done'
exit
p: procedure
  y = copies('x', 1000)
  r = >y
  return
