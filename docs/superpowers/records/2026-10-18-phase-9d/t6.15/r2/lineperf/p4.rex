/* LINEIN, then the read position by lines. */
parse arg f k
do i = 1 to k
  l = linein(f)
  q = stream(f, 'c', 'query position read line')
end
say q
