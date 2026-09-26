/* TRACE C over the ADDRESS instruction's own command against a command
   clause: no extension. */
trace c
address system "exit 3"
address nosuch "x"
"exit 0"
