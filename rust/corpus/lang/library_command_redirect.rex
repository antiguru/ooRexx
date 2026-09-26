/* orxfunction's I/O handler reading and writing each stream through the
   redirector context, in the FUNCTION group's shapes: stems replaced and
   appended, arrays, an output stream object, a file, buffered writes split into lines,
   a buffered read, a target that is also the source, and an environment's
   standing WITH configuration. */
call TestAddCommandEnvironment "io", "redirecting"

address io 'INPUTOUTPUT' with input using "This is a test"
say 'dropped' rc
address io 'INPUTOUTPUT' with input using "This is a test" output stem a.
say 'stem' rc a.0 a.1
address io 'INPUTOUTPUT' with input using "another" output replace stem a.
say 'replace' rc a.0 a.1
address io 'INPUTOUTPUT' with input using "fourth" output append stem a.
say 'append' rc a.0 a.1 a.2
address io 'INPUTOUTPUT' with input using "fifth" output append stem b.
say 'append new' rc b.0 b.1
address io 'INPUTERROR' with input using "err" error stem e.
say 'error' rc e.0 e.1
address io 'INPUTERROR' with input using "err2" error append stem e.
say 'error append' rc e.0 e.1 e.2
address io 'INPUTBOTH' with input using (('l1', 'l2', 'l3')) output stem o. error stem e.
say 'both' rc o.0 o.1 o.2 '|' e.0 e.1
address io 'INPUTBOTH' with input using (('l1', 'l2', 'l3')) output stem s. error stem s.
say 'both same' rc s.0 s.1 s.2 s.3

in.0 = 1; in.1 = "from a stem"
array = .array~new
address io 'INPUTOUTPUT' with input stem in. output using (array)
say 'array' rc array~items array[1]
in.1 = "again"
address io 'INPUTOUTPUT' with input stem in. output append using (array)
say 'array append' rc array~items array[1] array[2]

outstream = .ArrayOutputStream~new
address io 'INPUTOUTPUT' with input using (("Line1", "Line2")) output using (outstream)
say 'stream object' rc outstream~array~items outstream~array[1] outstream~array[2]

call lineout 'cmdinput.txt', 'Line1'
call lineout 'cmdinput.txt', ''
call lineout 'cmdinput.txt', 'Line3'
call lineout 'cmdinput.txt'
array = .array~new
address io 'INPUTOUTPUT' with input stream 'cmdinput.txt' output using (array)
say 'file' rc array~items array[1] '['array[2]']' array[3]
address io 'NOBLANKOUTPUT' with input stream 'cmdinput.txt' output stream 'cmdinput.txt'
say 'file same' rc lines('cmdinput.txt') linein('cmdinput.txt') linein('cmdinput.txt')
call stream 'cmdinput.txt', 'c', 'close'
call sysfiledelete 'cmdinput.txt'

a.0 = 3; a.1 = "Line1"; a.2 = ""; a.3 = "Line3"
address io 'NOBLANKOUTPUT' with input stem a. output stem a.
say 'stem same' rc a.0 a.1 a.2
address io 'NOBLANKOUTPUT' with input stem a. output append stem a.
say 'stem same append' rc a.0 a.1 a.2 a.3 a.4
address io 'NOBLANKERROR' with input using (('x', '', 'y')) error stem e.
say 'noblank error' rc e.0 e.1 e.2

crlf = '0d0a'x; lf = '0a'x; cr = '0d'x
array = .array~new
call buffered "Line1" || crlf
call buffered ("Line1", "Line2")
call buffered "Line1" || crlf || "Line2"
call buffered ("Line1" || cr, lf || "Line2")
call buffered "Line3" || cr || "Line4"
call buffered ("Line1" || cr, "Line2")
call buffered ("Line1" || crlf || "Line2" || crlf || "Li", "ne3" || crlf || "Line4")
call buffered (crlf || "Line1" || crlf || crlf || "Line2" || cr, lf || "Line3")
address io 'BUFFERERROR' with input using ("Line5" || lf || "Line6") error using (array)
say 'buffer error' rc array~items array[1] array[2]

drop a. b.
a.0 = 3; a.1 = "Line1"; a.2 = ""; a.3 = "Line3"
address io 'BUFFERINPUT' with input using (a.) output using (b.)
say 'buffered input' rc b.0 b.1 '['b.2']' b.3
address io 'BUFFERINPUT' with output stem b.
say 'buffered nothing' rc b.0

signal off novalue
drop a. b.
address io with input using "standing" output stem a. error stem b.
address io 'INPUTOUTPUT'
say 'standing' rc a.0 a.1 b.0
drop a. b.
'INPUTERROR'
say 'standing error' rc b.0 b.1 a.0
drop a. b.
address io 'INPUTOUTPUT' with input using 'override'
say 'override input' a.0 a.1 b.0
drop a. b.
address io 'INPUTOUTPUT' with output stem c.
say 'override output' c.0 c.1 a.0 b.0
drop a. b. c.
address io 'INPUTOUTPUT' with input normal
say 'normal input' a.0 b.0
drop a. b.
address io 'INPUTOUTPUT' with error normal
say 'normal error' a.0 a.1 b.0
address command
address
drop a. b.
'INPUTOUTPUT'
say 'toggled back' a.0 a.1 b.0
exit

buffered:
  address io 'BUFFEROUTPUT' with input using (arg(1)) output using (array)
  line = 'buffer' rc array~items
  do item over array
    line = line c2x(item)
  end
  say line
  return

::routine TestAddCommandEnvironment PUBLIC EXTERNAL "LIBRARY orxfunction"

::class ArrayOutputStream public inherit OutputStream
::method init
  expose array
  array = .array~new
::method lineout
  expose array
  use arg line
  array~append(line)
  return 0
::method charout
  raise syntax 93.963
::method array
  expose array
  return array
