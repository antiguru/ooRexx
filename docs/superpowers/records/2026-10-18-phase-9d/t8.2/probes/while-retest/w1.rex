do i = 1 while f(i)
  nop
end
say 'end1'
do j = 1 until g(j)
  if j = 1 then iterate
  nop
end
say 'end2'
k = 0
do while h(k)
  k = k + 1
end
exit
f: procedure
  use arg n
  say 'f' n sigl .context~stackframes[2]~line
  return n < 3
g: procedure
  use arg n
  say 'g' n sigl .context~stackframes[2]~line
  return n > 2
h: procedure
  use arg n
  say 'h' n sigl .context~stackframes[2]~line
  if n = 2 then return 1/0
  return 1
