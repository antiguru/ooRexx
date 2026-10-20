/* SEEK to each line in turn, then LINEIN. */
parse arg f k
call stream f, 'c', 'open read'
do i = 1 to k
  call stream f, 'c', 'seek' i 'read line'
  l = linein(f)
end
say i - 1
