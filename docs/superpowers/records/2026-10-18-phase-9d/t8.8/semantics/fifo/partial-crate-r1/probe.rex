f = '/tmp/t88f-fifo/partial-crate-r1/pipe'
log = '/tmp/t88f-fifo/partial-crate-r1/events'
call on halt name h
call on notready name nr
say 'warm ['charin(f,,3)']'
call lineout '/tmp/t88f-fifo/partial-crate-r1/ready', 'READY'
call stream '/tmp/t88f-fifo/partial-crate-r1/ready', 'c', 'close'
x = charin(f,,2)
say 'read [' || x || ']'
say 'description' stream(f,'d')
exit 0
nr:
say 'NOTREADY' condition('d') stream(f,'d')
call lineout log, 'NOTREADY 'stream(f,'d')
call stream log, 'c', 'close'
return
h:
say 'HALT' stream(f,'d')
call lineout log, 'HALT 'stream(f,'d')
call stream log, 'c', 'close'
return
