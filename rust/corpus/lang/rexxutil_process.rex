/* SysUtilVersion, SysLoadFuncs, SysDropFuncs, SysQueryProcess,
   SysSetPriority, SysCls, SysGetKey and SysCreatePipe. Process numbers,
   times and descriptors differ from run to run, so each is shown by how it
   relates to /proc. A command sends a line through a pipe by the inherited
   descriptors, from two activities at once. */
say '-- version and the no-ops'
say SysUtilVersion() (SysUtilVersion() == .RexxInfo~version)
say SysLoadFuncs() SysDropFuncs()
call SysLoadFuncs
say result
call t "SysUtilVersion(1)"
call t "SysLoadFuncs('x')"
call t "SysDropFuncs(, )"
call t "SysCls(1)"
call t "SysWait(1)"
call t "SysFork(1)"

say '-- SysQueryProcess'
stat = statFields('/proc/self/stat')
say 'pid' (SysQueryProcess() = stat~pid) (SysQueryProcess('pid') = stat~pid) (SysQueryProcess('Pid'||'00'x||'x') = stat~pid)
say 'ppid' (SysQueryProcess('PPID') = stat~ppid)
parent = statFields('/proc/'stat~ppid'/stat')
say 'pgid of the parent' (SysQueryProcess('pgid') = parent~group)
say 'pprio' (SysQueryProcess('PPRIO') = statFields('/proc/thread-self/stat')~nice)
say translate(SysQueryProcess('PTIME'), '9999999999', '0123456789')
parse value SysQueryProcess('pmem') with label size
say label datatype(size, 'W') (size > 0)
say SysQueryProcess('PSWAPS')
say SysQueryProcess('prcvdsig')
call t "SysQueryProcess('bogus')"
call t "SysQueryProcess('')"
call t "SysQueryProcess('PID', 1)"
call t "SysQueryProcess(.array~new)"

say '-- SysSetPriority'
nice = SysQueryProcess('PPRIO')
if nice < 19 then do
  say SysSetPriority(0, -(nice + 1)) (SysQueryProcess('PPRIO') - nice)
  say 'thread' (statFields('/proc/thread-self/stat')~nice - nice)
  say SysSetPriority(, -(nice + 1)) (SysQueryProcess('PPRIO') - nice)
end
say SysSetPriority(, -30) SysQueryProcess('PPRIO')
call t "SysSetPriority()"
call t "SysSetPriority(1)"
call t "SysSetPriority('a')"
call t "SysSetPriority('a', 0)"
call t "SysSetPriority(, 'a')"
call t "SysSetPriority(, 1.5)"
call t "SysSetPriority(, 2147483648)"
call t "SysSetPriority(1, 2, 3)"

say '-- SysCls'
call value 'TERM', 'xterm', 'ENVIRONMENT'
say 'before'
r = SysCls()
say 'after' r
call value 'TERM', 'dumb', 'ENVIRONMENT'
say 'dumb' SysCls()
call value 'TERM', 'nosuchterm', 'ENVIRONMENT'
say 'unknown' SysCls()
path = value('PATH', , 'ENVIRONMENT')
call value 'PATH', '/nonexistent', 'ENVIRONMENT'
say 'no clear' SysCls()
call value 'PATH', path, 'ENVIRONMENT'

say '-- SysGetKey, standard input not a terminal'
say '['SysGetKey()']' '['SysGetKey('noecho')']' '['SysGetKey('ECHO')']' '['SysGetKey(,)']'
call t "SysGetKey('bogus')"
call t "SysGetKey('')"
call t "SysGetKey('echo', 1)"

say '-- SysCreatePipe'
call pipe SysCreatePipe(), 'default'
call pipe SysCreatePipe('b'), 'b'
call pipe SysCreatePipe('Blocking'), 'Blocking'
call pipe SysCreatePipe('n'), 'n'
call pipe SysCreatePipe('Nonblocking'||'00'x||'x'), 'Nonblocking'
call t "SysCreatePipe('')"
call t "SysCreatePipe('x')"
call t "SysCreatePipe('00'x||'n')"
call t "SysCreatePipe('b', 1)"

say '-- two activities at once'
a = .sender~new~start('send', 'first')
b = .sender~new~start('send', 'second')
say a~result
say b~result

say '-- an untrapped error'
say SysQueryProcess('PTIMES')
exit

t:
  parse arg expr
  signal on syntax name trap
  interpret 'r = ' expr
  say expr '->' r
  return
trap:
  say expr '->' condition('O')~code condition('O')~message
  return

/* A process's /proc stat line as a directory of the fields this program
   compares: proc(5) fields 1, 4, 5 and 19. */
statFields: procedure
  parse arg name
  line = linein(name)
  call stream name, 'c', 'close'
  after = substr(line, lastpos(')', line) + 1)
  fields = .directory~new
  fields~pid = word(line, 1)
  fields~ppid = word(after, 2)
  fields~group = word(after, 3)
  fields~nice = word(after, 17)
  return fields

/* The flags line of a descriptor's /proc fdinfo. */
fdFlags: procedure
  parse arg fd
  name = '/proc/self/fdinfo/'fd
  flags = '?'
  do 10
    parse value linein(name) with key ':' found
    if key == 'flags' then flags = strip(found)
  end
  call stream name, 'c', 'close'
  return flags

/* A pipe's two descriptors and their flags. */
pipe: procedure
  parse arg ends, label
  parse var ends reading writing extra
  say label words(ends) datatype(reading, 'W') datatype(writing, 'W') (reading <> writing)
  say label 'flags' fdFlags(reading) fdFlags(writing)
  return

::class sender
::method send
  use arg text
  parse value SysCreatePipe() with reading writing
  command = 'printf "%s\n" "'text' through a pipe" > /proc/self/fd/'writing'; head -n 1 /proc/self/fd/'reading
  address system command with output stem out.
  return rc out.0 out.1
