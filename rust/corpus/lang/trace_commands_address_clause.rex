/* TRACE C echoes an ADDRESS clause that issues a command, as it does a
   command clause, and not one that only sets the environment. */
trace c
address system 'echo a' with output stem o.
address system 'echo b'
'echo c'
address system
trace o
say o.0 o.1
