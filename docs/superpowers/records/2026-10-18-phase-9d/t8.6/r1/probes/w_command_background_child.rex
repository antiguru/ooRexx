/* A command completes when its process ends, so a background process it
   leaves running, holding the command's output open, does not keep the
   clause waiting. Output redirected to a target is read to its end. */
address system 'mkfifo ff1'
address system '(sleep 0.3; printf abc > ff1) &'
v = charin('ff1',,3)
say 'v=['v']' stream('ff1','S')
call stream 'ff1', 'c', 'close'
call time 'R'
address system 'echo now; (sleep 1) &'
say 'not waited' rc (time('E') < 0.8)
call time 'R'
address system 'echo out; (sleep 1; echo late) &' with output stem s.
say 'redirected:' s.0 s.1 s.2 (time('E') > 0.8)
call time 'R'
address system '(sleep 1; echo late 1>&2) &' with error stem e.
say 'redirected error:' e.0 e.1 (time('E') > 0.8)
address system 'rm -f ff1'
