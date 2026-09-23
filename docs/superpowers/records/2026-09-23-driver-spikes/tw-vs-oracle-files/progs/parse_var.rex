n = 200000; y = 5; x = 0; a = 'abc'; b = 'abd'; s = 'one two three'
do i = 1 to n
  parse var s a b c
end
say x
exit
r: return
