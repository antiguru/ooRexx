say 'a'
address system 'echo c1' with output stream 'STDOUT'
say 'b'
address system 'echo c2' with output using (.stdout)
say 'c'
o = .stream~new('STDOUT')
address system 'echo c3' with output using (o)
say 'd'
address system 'echo c4 >&2' with error stream 'STDERR'
call lineout 'STDERR', 'e'
o~close
say 'f'
