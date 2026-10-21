f = .flag~new
.setter~new(f)~start('GO')
.traceoutput~destination(.dst~new(f))
trace r
x = 1
trace off
say 'done' f~v
::class flag; ::attribute v
::class setter
::method init; expose f; use arg f
::method go; expose f; call syssleep 0.2; f~v = 1
::class dst
::method init; expose f; use arg f
::method lineout; expose f; do while f~v \= 1; nop; end
