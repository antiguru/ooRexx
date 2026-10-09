/* CALL ON NOTREADY runs its handler in the program for a read of standard
   input at its end, through LINEIN, .STDIN~LINEIN and CHARIN. */
call on notready name h
x = linein()
say 'x=' x
y = .stdin~linein
say 'y=' y
z = charin()
say 'z=' z
exit
h: say 'h sigl' sigl condition('D') condition('I'); return
