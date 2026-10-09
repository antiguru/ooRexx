/* A debug pause with .DEBUGINPUT set to .nil sends LINEIN to .nil, and the
   error is reported against the clause that paused. */
.local~debuginput = .nil
trace ?a
say 'x'
say 'y'
