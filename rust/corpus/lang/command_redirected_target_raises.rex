/* An output target that raises while its command still runs ends that
   command's clause with the condition, and the next redirected command's
   wait is its own: it gets its own child's output and return code. */
signal on syntax name raised
address system 'echo a; sleep 0.3; echo b' with output using (.Bad~new)
say 'not here'
raised:
say 'trapped' condition('o')~code
signal off syntax
address system 'sleep 0.8; echo c; echo d' with output stem o.
say 'next' rc o.0 o.1 o.2
exit

::class Bad subclass OutputStream
::method lineout
  return 1 / 0
