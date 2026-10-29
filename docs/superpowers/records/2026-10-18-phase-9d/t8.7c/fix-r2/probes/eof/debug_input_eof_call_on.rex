/* At the end of standard input each debug pause raises NOTREADY, which a
   CALL ON trap runs in the program with the paused clause as SIGL. */
call on notready name nr
trace ?a
say 'a'
say 'b'
exit
nr: say 'nr' sigl; return
