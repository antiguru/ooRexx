/* With .DebugInput set to .stdin the NOTREADY of each pause runs its CALL ON
   handler with the paused clause as SIGL. */
.local~debuginput = .stdin
call on notready name nr
trace ?a
say 'a'
say 'b'
exit
nr: say 'nr' sigl; return
