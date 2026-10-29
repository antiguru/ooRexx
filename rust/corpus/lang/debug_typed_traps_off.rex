/* Traps are OFF while a line typed at a pause runs: a NOVALUE, a USER and a
   NOTREADY raised there take their default actions. The pause's own read of
   standard input is an ordinary read: its NOTREADY at the end is trapped. */
signal on novalue name hv
call on user ping name hu
signal on notready name hn
trace ?a
x = 1
say 'after' x
exit
hv: say 'hv' condition('D') sigl; exit
hu: say 'hu' condition('D') sigl; return
hn: say 'hn' condition('D') sigl; exit
::routine ru
  raise user ping description 'from ru' return 5
