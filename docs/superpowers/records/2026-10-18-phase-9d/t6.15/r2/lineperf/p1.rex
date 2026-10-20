/* LINES (Count, the method's default) as a loop condition, LINEIN in it. */
parse arg f .
s = .stream~new(f)
zz = s~open('read')
n = 0
do while s~lines > 0
  l = s~linein
  n = n + 1
end
say n
