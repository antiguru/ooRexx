/* A debug pause with no .DEBUGINPUT entry reads a null string and leaves
   standard input to PULL. */
.local~remove('DEBUGINPUT')
trace ?a
say 'x'
parse pull q
say 'pulled' q
