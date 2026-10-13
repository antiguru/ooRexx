/* stream builtins resolving a name per call */
f = 'cg_stream.txt'
do i = 1 to 20000
  call lineout f, 'line' i
end
call lineout f
do i = 1 to 20000
  x = linein(f)
end
n = lines(f)
call stream f, 'c', 'close'
say x n
