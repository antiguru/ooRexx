/* LINEIN naming each line in turn. */
parse arg f k
do i = 1 to k
  l = linein(f, i)
end
say i - 1
