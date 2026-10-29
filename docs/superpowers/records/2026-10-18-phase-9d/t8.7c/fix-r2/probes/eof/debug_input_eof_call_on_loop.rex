/* The NOTREADY a debug pause raises at the end of input is delivered after
   the DO header that paused, before the body's first clause. */
call on notready name nr
trace ?a
do i = 1 to 2
  say i
end
'true'
exit
nr: say 'nr' sigl; return
