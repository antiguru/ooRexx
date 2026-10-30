f = '/tmp/t88f-fifo/oracle-r4/pipe'
log = '/tmp/t88f-fifo/oracle-r4/events'
call on halt name h
call on notready name nr
say 'warm ['charin(f,,3)']'
call lineout '/tmp/t88f-fifo/oracle-r4/ready', 'READY'
call stream '/tmp/t88f-fifo/oracle-r4/ready', 'c', 'close'
x = charin(f,,1)
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
