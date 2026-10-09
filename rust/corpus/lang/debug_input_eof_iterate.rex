/* At the end of input, the NOTREADY of the pause after an ITERATE runs its
   CALL ON handler at that pause, with the ITERATE as SIGL. */
call on notready name nr
trace ?a
do i = 1 to 2
  iterate
end
say 'after'
exit
nr: say 'nr' sigl; return
