/* `ADDRESS ... WITH STREAM`, and the stream objects a `USING` redirection
   drives.

   A named stream is opened by the redirection and closed by it; a stream
   object handed over by `USING` is written and left open, because the program
   owns it. The same file may be both the input and the output: everything the
   command reads is gathered before the output side opens.

   Two names for one file are one target -- the comparison is on the qualified
   name, so `both.txt` and `./both.txt` collapse and the two streams interleave
   in the order the command wrote them.

   Every file this program touches it creates in its own directory. It reads
   them back a line at a time rather than by counting, because a stream's line
   count is cached per object and a second handle to the same path answers
   from its own cache. */

address sh with output stream 'out.txt'
'printf "a\nb\nc\n"'
say 'written'
call show 'out.txt'

address sh with output append stream 'out.txt'
'echo d'
say 'appended'
call show 'out.txt'

address sh 'cat' with input stream 'out.txt' output stream 'out.txt'
say 'round trip'
call show 'out.txt'

address sh 'sh -c "echo o1; echo e1 1>&2; echo o2"' with output stream 'both.txt' error stream './both.txt'
say 'one file, two streams'
call show 'both.txt'

held = .stream~new('obj.txt')
address sh 'printf "m1\nm2\n"' with output using (held)
say 'object state' held~state
say 'object close' held~close
say 'object wrote'
call show 'obj.txt'

source = .stream~new('out.txt')
address sh 'cat' with input using (source) output stem got.
say 'from an object' got.0 got.1
call stream 'out.txt', 'c', 'close'

signal on syntax name noread
address sh 'cat' with input stream 'nosuch.txt'
say 'not reached'

noread:
say 'unreadable' rc
signal on syntax name nowrite
address sh 'echo z' with output stream 'nodir/out.txt'
say 'not reached'

nowrite:
say 'unwritable' rc
signal on syntax name optioned
address sh 'echo z' with output append using (.stream~new('so.txt'))
say 'not reached'

optioned:
say 'bad option' rc
exit

show: procedure
  use arg name
  do while lines(name) > 0
    say '  [' || linein(name) || ']'
  end
  call stream name, 'c', 'close'
  return
