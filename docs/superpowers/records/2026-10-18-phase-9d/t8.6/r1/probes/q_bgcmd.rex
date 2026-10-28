address system 'mkfifo ff1'
address system '(sleep 0.3; printf abc > ff1) &'
v = charin('ff1',,3)
say 'v=['v']' stream('ff1','S')
