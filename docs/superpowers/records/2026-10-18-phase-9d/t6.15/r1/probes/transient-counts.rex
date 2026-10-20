/* CHARS and LINES on /dev/null and on a FIFO, empty and holding data. */
n = 0
do while lines('/dev/null') > 0 & n < 5
  n = n + 1
  l = linein('/dev/null')
end
say 'devnull: chars' chars('/dev/null') 'lines' lines('/dev/null') 'loop ran' n
address system 'mkfifo ff'
call stream 'ff', 'c', 'open both nobuffer'
call charout 'ff', 'abcd'
call lineout 'ff', 'e'
say 'fifo with data: chars' chars('ff') 'lines' lines('ff') lines('ff', 'C')
say 'then charin' charin('ff', , 2) 'linein' linein('ff')
