/* NEW and NEWFILE on Method and Routine subclasses: the compiled executable
   becomes an instance of the subclass and INIT gets the arguments after the
   context; every Method and Routine method works on it; it is not a Method
   or Routine where the language asks for one (CALL, DEFINE, SETMETHOD and
   RUN refuse it) while ~addRoutine admits it; COPY keeps its source.
   LOADEXTERNAL* sent to a subclass answers the base class's object, and a
   base NEW given an argument past the context sends INIT with it. */
m = .km~new('m', 'return 42')
say m~class~id m~source~items m~source[1] m~isGuarded m~isPrivate m~package~class~id
say m~scope m~annotations~class~id (m~annotations == m~annotations) m~annotation('x')
m~setPrivate; say m~isPrivate
m~setUnguarded; say m~isGuarded
c = m~copy; say c~class~id c~source[1] c~isPrivate c~isGuarded
say m~objectName m~string
say m~isProtected m~isAbstract m~isAttribute m~isConstant
m~setProtected; say m~isProtected
m~setGuarded; say m~isGuarded
r = .kr~new('r', 'return arg(1) + 1')
say r~class~id r~call(5) r~callWith(.array~of(9)) r~source[1] r[1]
say r~package~class~id r~annotations~class~id r~setSecurityManager(.nil)
p = .context~package
p~addRoutine('RR', r); say p~routines['RR']~class~id rr(3)
r2 = .kr~new('r2', 'return 1', .context~package); say r2~call
t = .table~new; t[m] = 1; say t[m] (t[m~copy] == .nil)
say r~copy~class~id r~copy~call(1)
dm = .km~new('dm', .array~of('return 5', '::routine helper', 'return 6'))
say dm~class~id dm~package~routines~hasIndex('HELPER')
dr = .kr~new('dr', .array~of('return helper2()', '::routine helper2', "return 'h2'"))
say dr~class~id dr~call
i = .kmi~new('i', 'return 1', .context~package, 'x', 'y'); say i~class~id
i = .kri~new('i', 'return 1'); say i~class~id
f = 'executable_subclass_new.tmp'
call lineout f, 'return arg(1) * 2'
call lineout f
nf = .kri~newFile(f); say nf~class~id nf~call(21) nf~source[1]
nf = .kmi~newFile(f); say nf~class~id nf~source~items
call sysFileDelete f
say .km~loadExternalMethod('x', 'LIBRARY REXX file_separator')~class~id
say .kr~loadExternalRoutine('y', 'LIBRARY nosuchlib zz')
signal on syntax name e1
call (r)
e1: say 'e1' condition('O')~code condition('O')~message
signal on syntax name e2
.zz~define('Q', m)
e2: say 'e2' condition('O')~code condition('O')~message
signal on syntax name e3
say .hm~new~try(m)
e3: say 'e3' condition('O')~code condition('O')~message
signal on syntax name e4
say .hm~new~go(.km~new('g', 'return 1'))
e4: say 'e4' condition('O')~code condition('O')~message
signal on syntax name e5
x = .km~new('m')
e5: say 'e5' condition('O')~code condition('O')~message
signal on syntax name e6
x = .km~new('m', 'return 1', 'bogus')
e6: say 'e6' condition('O')~code condition('O')~message
signal on syntax name e7
x = .ka~new('a', 'return 1')
e7: say 'e7' condition('O')~code condition('O')~message
signal on syntax name e8
x = .method~new('test', 'nop', 'programscope', '')
e8: say 'e8' condition('O')~code condition('O')~message
signal on syntax name e9
x = .routine~new('test', 'nop', 'programscope', '')
e9: say 'e9' condition('O')~code condition('O')~message
exit
::class km subclass method
::class kr subclass routine
::class ka subclass method abstract
::class kmi subclass method
::method init
  say 'init' arg() arg(1) arg(2)
  self~init:super
::class kri subclass routine
::method init
  say 'rinit' arg()
::class zz
::class hm
::method try
  use arg m
  self~setMethod('HM', m)
  return self~hm
::method go
  use arg m
  return self~run(m)
