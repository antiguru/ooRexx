'mkfifo ff'
'(cat ff > got.txt) &'
call lineout 'ff', 'via fifo'
call SysSleep 0.5
say 'before close' chars('got.txt')
call lineout 'ff'
call SysSleep 0.5
say 'after close' chars('got.txt')
'rm -f ff'
