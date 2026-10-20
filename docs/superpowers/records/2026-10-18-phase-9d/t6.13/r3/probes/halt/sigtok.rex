signal on halt
address system 'echo a; sleep 1.5; echo LATE' with output stem s.
say 'not here'
exit
halt:
  say 'HALT rc' rc 'stem' s.0 s.1
  address system 'sleep 2; echo x' with output stem t.
  say 'second rc' rc 't' t.0 t.1 t.2
  exit 0
