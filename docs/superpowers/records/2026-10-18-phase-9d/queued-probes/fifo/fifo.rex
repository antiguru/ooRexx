address system 'mkfifo ff1'
address system '(sleep 0.3; printf abc > ff1) </dev/null >/dev/null 2>&1 &'
v = charin('ff1',,3)
say 'v=['v']' stream('ff1','S')
